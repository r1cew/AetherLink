/// AetherLink Phone — Tauri-команды для Android UI.
///
/// ┌─────────────────────────────┬─────────────────────────────────────────────┐
/// │ Команда                      │ Назначение                                  │
/// ├─────────────────────────────┼─────────────────────────────────────────────┤
/// │ pair_with_qr(qr, name)      │ Привязать ПК по QR-коду                     │
/// │ get_servers()               │ Список привязанных ПК                       │
/// │ remove_server(id)           │ Удалить ПК                                  │
/// │ send_safe(id, cmd, params)  │ Safe Mode — системная команда               │
/// │ send_run_profile(id, pid)   │ Automation — запустить профиль              │
/// │ list_profiles(id)           │ Automation — список профилей с ПК           │
/// │ send_shell(id, cmd, shell)  │ Developer — выполнить shell-команду         │
/// │ discover_and_update(id)     │ Найти ПК по beacon (если IP сменился)       │
/// └─────────────────────────────┴─────────────────────────────────────────────┘
use std::{path::PathBuf, sync::Arc};

use tauri::Manager;
use tokio::sync::Mutex;
use uuid::Uuid;

use aetherlink_common::beacon;
mod connection;
mod keypair;
mod servers;

use aetherlink_common::protocol::{ClientRequest, SafeCommand, ShellType};
use keypair::PhoneKeypair;
use servers::{load as load_servers, save as save_servers, SavedServer};

// ─── Состояние приложения ─────────────────────────────────────────────────────

struct AppStateInner {
    data_dir: PathBuf,
    keypair: PhoneKeypair,
    /// Хранилище активного долгоживущего соединения: (server_id, сессия сокета)
    session: Arc<std::sync::Mutex<Option<(String, connection::ActiveSession)>>>,
}
type AppState = Arc<Mutex<AppStateInner>>;

// ─── QR данные от ПК ─────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct QrData {
    ip: String,
    port: u16,
    server_public_key: String,
    pairing_token: String,
}

// ─── Вспомогательная: отправить запрос через постоянную сессию ────────────────

async fn send_with_fallback(
    state: &AppState,
    server_id: &str,
    request: ClientRequest,
) -> Result<serde_json::Value, String> {
    let (data_dir, keypair, session_arc) = {
        let s = state.lock().await;
        (s.data_dir.clone(), s.keypair.clone(), s.session.clone())
    };

    // Сериализуем запрос в строку, чтобы безопасно передать через границу потоков spawn_blocking
    let req_json = serde_json::to_string(&request).unwrap();

    // Попытка 1: Отправляем через живую сессию (или создаем её, если пустая)
    let result = tauri::async_runtime::spawn_blocking({
        let server_id = server_id.to_string();
        let data_dir = data_dir.clone();
        let keypair = keypair.clone();
        let req_json = req_json.clone();
        let session_arc = session_arc.clone();

        move || {
            let mut lock = session_arc.lock().unwrap();

            // Если сессия открыта для ДРУГОГО сервера, сбрасываем её
            let is_matching = match &*lock {
                Some((id, _)) => id == &server_id,
                None => false,
            };
            if !is_matching {
                *lock = None;
            }

            // Если сокет еще не создан — подключаемся и делаем хендшейк (ОДИН РАЗ)
            if lock.is_none() {
                let servers = load_servers(&data_dir)?;
                let server = servers
                    .iter()
                    .find(|s| s.id == server_id)
                    .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

                let new_sess = connection::ActiveSession::new(server, &keypair)?;
                *lock = Some((server_id.clone(), new_sess));
            }

            // Извлекаем сессию и пушим команду в трубу
            let (_, session) = lock.as_mut().unwrap();
            let req: ClientRequest = serde_json::from_str(&req_json).unwrap();

            match session.send_request(&req) {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    // При любой сетевой ошибке (таймаут, разрыв) уничтожаем сессию,
                    // чтобы fallback-логика или следующий клик построили сокет заново
                    *lock = None;
                    Err(e)
                }
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    // Если всё улетело успешно по постоянному каналу — сразу отдаем фронтенду
    match result {
        Ok(resp) => return response_to_value(resp),
        Err(_) => {
            println!("[client] Сессия мертва или IP изменился. Запуск авто-поиска по beacon...")
        }
    }

    // Попытка 2 (Fallback): сокет упал, возможно у ПК изменился IP-адрес. Ищем по beacon.
    let new_ip = tauri::async_runtime::spawn_blocking(|| beacon::discover_client(15))
        .await
        .map_err(|e| e.to_string())??;

    let mut servers = load_servers(&data_dir)?;
    let idx = servers
        .iter()
        .position(|s| s.id == server_id)
        .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

    servers[idx].ip = new_ip;
    save_servers(&data_dir, &servers)?;

    // Создаем новую сессию по свежему IP и отправляем команду
    let resp = tauri::async_runtime::spawn_blocking({
        let server_id = server_id.to_string();
        let data_dir = data_dir.clone();
        let keypair = keypair.clone();
        let req_json = req_json;
        let session_arc = session_arc;

        move || {
            let mut lock = session_arc.lock().unwrap();
            *lock = None; // Гарантированно чистим старый сокет

            let servers = load_servers(&data_dir)?;
            let server = servers
                .iter()
                .find(|s| s.id == server_id)
                .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

            let mut new_sess = connection::ActiveSession::new(server, &keypair)?;
            let req: ClientRequest = serde_json::from_str(&req_json).unwrap();
            let res = new_sess.send_request(&req);

            if res.is_ok() {
                *lock = Some((server_id, new_sess));
            }
            res
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    response_to_value(resp)
}

fn response_to_value(
    resp: aetherlink_common::protocol::ServerResponse,
) -> Result<serde_json::Value, String> {
    if resp.ok {
        Ok(resp
            .data
            .unwrap_or_else(|| serde_json::Value::String(resp.output.unwrap_or_default())))
    } else {
        Err(resp.error.unwrap_or_else(|| "Ошибка сервера".into()))
    }
}

// ───  Привязать новый ПК по QR-коду ────────────────────────────────────────────
#[tauri::command]
async fn pair_with_qr(
    state: tauri::State<'_, AppState>,
    qr_json: String,
    name: String,
    nickname: String,
) -> Result<String, String> {
    let qr: QrData = serde_json::from_str(&qr_json).map_err(|e| format!("Неверный QR: {e}"))?;

    let (data_dir, keypair) = {
        let s = state.inner().lock().await;
        (s.data_dir.clone(), s.keypair.clone())
    };

    let temp = SavedServer {
        id: "_pairing_".into(),
        name: nickname.clone(),
        ip: qr.ip.clone(),
        port: qr.port,
        server_public_key_b64: qr.server_public_key.clone(),
    };

    let request = ClientRequest::Pair {
        token: qr.pairing_token,
        name,
    };

    // Для паринга используем разовую сессию ActiveSession
    let response = tauri::async_runtime::spawn_blocking(move || {
        let mut session = connection::ActiveSession::new(&temp, &keypair)?;
        session.send_request(&request)
    })
    .await
    .map_err(|e| e.to_string())??;

    if !response.ok {
        return Err(response.error.unwrap_or_else(|| "Паринг не удался".into()));
    }

    // Сохраняем сервер
    let server = SavedServer {
        id: Uuid::new_v4().to_string(),
        name: nickname,
        ip: qr.ip,
        port: qr.port,
        server_public_key_b64: qr.server_public_key,
    };
    let server_id = server.id.clone();

    let mut existing = load_servers(&data_dir)?;
    existing.push(server);
    save_servers(&data_dir, &existing)?;

    Ok(server_id)
}

// ───  Работа с привязанными ПК ────────────────────────────────────────────
/// Список привязанных ПК (без приватных ключей).
#[tauri::command]
async fn get_servers(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let data_dir = state.inner().lock().await.data_dir.clone();
    let servers = load_servers(&data_dir)?;
    let list: Vec<_> = servers
        .iter()
        .map(|s| {
            serde_json::json!({
                "id":   s.id,
                "name": s.name,
                "ip":   s.ip,
                "port": s.port,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(list))
}

/// Удалить привязанный ПК.
#[tauri::command]
async fn remove_server(state: tauri::State<'_, AppState>, server_id: String) -> Result<(), String> {
    let (data_dir, session_arc) = {
        let s = state.inner().lock().await;
        (s.data_dir.clone(), s.session.clone())
    };

    // Если удаляем текущий сервер, закрываем его сессию сокета
    if let Ok(mut lock) = session_arc.lock() {
        if let Some((id, _)) = &*lock {
            if id == &server_id {
                *lock = None;
            }
        }
    }

    let mut servers = load_servers(&data_dir)?;
    servers.retain(|s| s.id != server_id);
    save_servers(&data_dir, &servers)
}

// ─── Работа с Safe Mode ────────────────────────────────────────────

/// Safe Mode: отправить системную команду.
#[tauri::command]
async fn send_safe(
    state: tauri::State<'_, AppState>,
    server_id: String,
    command: String,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let cmd = match command.as_str() {
        "shutdown" => SafeCommand::Shutdown,
        "sleep" => SafeCommand::Sleep,
        "lock" => SafeCommand::Lock,
        "volume_up" => SafeCommand::VolumeUp,
        "volume_down" => SafeCommand::VolumeDown,
        "volume_set" => SafeCommand::VolumeSet,
        "media_play" => SafeCommand::MediaPlay,
        "media_pause" => SafeCommand::MediaPause,
        "media_next" => SafeCommand::MediaNext,
        "media_prev" => SafeCommand::MediaPrev,
        other => return Err(format!("Неизвестная команда: {other}")),
    };

    let request = ClientRequest::Safe {
        command: cmd,
        params: params.unwrap_or(serde_json::Value::Null),
    };

    send_with_fallback(state.inner(), &server_id, request).await
}

// ─── Работа с Automation Mode ────────────────────────────────────────────

/// Automation Mode: запустить профиль по ID.
#[tauri::command]
async fn send_run_profile(
    state: tauri::State<'_, AppState>,
    server_id: String,
    profile_id: String,
) -> Result<serde_json::Value, String> {
    send_with_fallback(
        state.inner(),
        &server_id,
        ClientRequest::RunProfile { profile_id },
    )
    .await
}

/// Automation Mode: получить список профилей с ПК.
#[tauri::command]
async fn list_profiles(
    state: tauri::State<'_, AppState>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    send_with_fallback(state.inner(), &server_id, ClientRequest::ListProfiles).await
}

// ─── Работа с Developer Mode ────────────────────────────────────────────

/// Developer Mode: выполнить команду в shell.
#[tauri::command]
async fn send_shell(
    state: tauri::State<'_, AppState>,
    server_id: String,
    cmd: String,
    shell: Option<String>,
) -> Result<serde_json::Value, String> {
    let shell_type = match shell.as_deref().unwrap_or("powershell") {
        "cmd" => ShellType::Cmd,
        _ => ShellType::Powershell,
    };
    send_with_fallback(
        state.inner(),
        &server_id,
        ClientRequest::Shell {
            cmd,
            shell: shell_type,
        },
    )
    .await
}

/// Developer Mode: проверить наличие dev статуса на ПК.
#[tauri::command]
async fn check_dev_status(
    state: tauri::State<'_, AppState>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    send_with_fallback(state.inner(), &server_id, ClientRequest::CheckDevStatus).await
}

/// Developer Mode: создать новый профиль с мобилки.
#[tauri::command]
async fn create_profile(
    state: tauri::State<'_, AppState>,
    server_id: String,
    name: String,
    description: Option<String>,
    commands: serde_json::Value,
) -> Result<serde_json::Value, String> {
    send_with_fallback(
        state.inner(),
        &server_id,
        ClientRequest::CreateProfile {
            name,
            description,
            commands,
        },
    )
    .await
}

/// Developer Mode: получить список профилей для редактирования.
#[tauri::command]
async fn get_dev_profiles(
    state: tauri::State<'_, AppState>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    send_with_fallback(state.inner(), &server_id, ClientRequest::GetDevProfiles).await
}

/// Developer Mode: удалить профиль.
#[tauri::command]
async fn delete_profile(
    state: tauri::State<'_, AppState>,
    server_id: String,
    profile_id: String,
) -> Result<serde_json::Value, String> {
    send_with_fallback(
        state.inner(),
        &server_id,
        ClientRequest::DeleteProfile { profile_id },
    )
    .await
}

// ─── beacon ────────────────────────────────────────────

/// Принудительно найти ПК через beacon и обновить его IP.
#[tauri::command]
async fn discover_and_update(
    state: tauri::State<'_, AppState>,
    server_id: String,
) -> Result<String, String> {
    let (data_dir, session_arc) = {
        let s = state.inner().lock().await;
        (s.data_dir.clone(), s.session.clone())
    };

    let new_ip = tauri::async_runtime::spawn_blocking(|| beacon::discover_client(20))
        .await
        .map_err(|e| e.to_string())??;

    let mut servers = load_servers(&data_dir)?;
    let server = servers
        .iter_mut()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

    server.ip = new_ip.clone();
    save_servers(&data_dir, &servers)?;

    // Обязательно сбрасываем закешированную сессию, чтобы сокет переподключился к новому IP
    if let Ok(mut lock) = session_arc.lock() {
        if let Some((id, _)) = &*lock {
            if id == &server_id {
                *lock = None;
            }
        }
    }

    Ok(new_ip)
}

// ─── Точка входа ──────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_store::Builder::new().build());

    builder
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Не удалось получить app data dir");
            std::fs::create_dir_all(&data_dir).expect("Не удалось создать директорию");

            let keypair =
                keypair::load_or_create(&data_dir).expect("Ошибка загрузки keypair телефона");

            // Инициализируем пустое состояние пула сессий при старте
            let state: AppState = Arc::new(Mutex::new(AppStateInner {
                data_dir,
                keypair,
                session: Arc::new(std::sync::Mutex::new(None)),
            }));
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pair_with_qr,
            get_servers,
            remove_server,
            send_safe,
            send_run_profile,
            list_profiles,
            send_shell,
            check_dev_status,
            create_profile,
            get_dev_profiles,
            delete_profile,
            discover_and_update,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска Tauri");
}
