/// AetherLink Phone — Tauri-команды для Android UI.
///
/// ┌─────────────────────────────┬─────────────────────────────────────────────┐
/// │ Команда                     │ Назначение                                  │
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

mod beacon;
mod connection;
mod keypair;
mod protocol;
mod servers;

use keypair::PhoneKeypair;
use protocol::{ClientRequest, SafeCommand, ShellType};
use servers::{load as load_servers, save as save_servers, SavedServer};

// ─── Состояние приложения ─────────────────────────────────────────────────────

struct AppStateInner {
    data_dir: PathBuf,
    keypair: PhoneKeypair,
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

// ─── Вспомогательная: отправить запрос с fallback на beacon ──────────────────

async fn send_with_fallback(
    state: &AppState,
    server_id: &str,
    request: ClientRequest,
) -> Result<serde_json::Value, String> {
    let (data_dir, keypair) = {
        let s = state.lock().await;
        (s.data_dir.clone(), s.keypair.clone())
    };

    let mut servers = load_servers(&data_dir)?;
    let idx = servers
        .iter()
        .position(|s| s.id == server_id)
        .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

    // Пробуем сохранённый IP
    let result = tauri::async_runtime::spawn_blocking({
        let server = servers[idx].clone();
        let keypair = keypair.clone();
        let req = serde_json::to_string(&request).unwrap();
        move || {
            let r: ClientRequest = serde_json::from_str(&req).unwrap();
            connection::send(&server, &keypair, &r)
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    match result {
        Ok(resp) => return response_to_value(resp),
        Err(_) => {} // IP устарел — ищем через beacon
    }

    // Fallback: beacon discovery
    let new_ip = tauri::async_runtime::spawn_blocking(|| beacon::discover(15))
        .await
        .map_err(|e| e.to_string())??;

    servers[idx].ip = new_ip;
    save_servers(&data_dir, &servers)?;

    let resp = tauri::async_runtime::spawn_blocking({
        let server = servers[idx].clone();
        let keypair = keypair.clone();
        let req = serde_json::to_string(&request).unwrap();
        move || {
            let r: ClientRequest = serde_json::from_str(&req).unwrap();
            connection::send(&server, &keypair, &r)
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    response_to_value(resp)
}

fn response_to_value(resp: protocol::ServerResponse) -> Result<serde_json::Value, String> {
    if resp.ok {
        Ok(resp
            .data
            .unwrap_or_else(|| serde_json::Value::String(resp.output.unwrap_or_default())))
    } else {
        Err(resp.error.unwrap_or_else(|| "Ошибка сервера".into()))
    }
}

// ─── Tauri-команды ────────────────────────────────────────────────────────────

/// Привязать новый ПК по QR-коду.
///
/// qr_json  — строка из QR (то что generate_pairing_qr() вернул на ПК).
/// name     — имя телефона (будет показано в списке устройств на ПК).
/// nickname — как называть этот ПК в списке на телефоне.
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

    // Временный SavedServer для паринга (без id — ещё не сохранён)
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

    let response =
        tauri::async_runtime::spawn_blocking(move || connection::send(&temp, &keypair, &request))
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
    let data_dir = state.inner().lock().await.data_dir.clone();
    let mut servers = load_servers(&data_dir)?;
    servers.retain(|s| s.id != server_id);
    save_servers(&data_dir, &servers)
}

/// Safe Mode: отправить системную команду.
///
/// command — строка: "shutdown"|"sleep"|"lock"|"volume_up"|"volume_down"|
///                   "volume_set"|"media_play"|"media_pause"|"media_next"|"media_prev"
/// params  — JSON (для volume_set: { "level": 50 })
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

/// Developer Mode: выполнить команду в shell.
/// shell — "powershell" | "cmd"
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

/// Принудительно найти ПК через beacon и обновить его IP.
/// Используется когда ПК поменял IP и обычное подключение не работает.
#[tauri::command]
async fn discover_and_update(
    state: tauri::State<'_, AppState>,
    server_id: String,
) -> Result<String, String> {
    let data_dir = state.inner().lock().await.data_dir.clone();

    let new_ip = tauri::async_runtime::spawn_blocking(|| beacon::discover(20))
        .await
        .map_err(|e| e.to_string())??;

    let mut servers = load_servers(&data_dir)?;
    let server = servers
        .iter_mut()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Сервер {server_id} не найден"))?;

    server.ip = new_ip.clone();
    save_servers(&data_dir, &servers)?;

    Ok(new_ip)
}

// ─── Точка входа ──────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(any(target_os = "android", target_os = "ios"))]
    let builder = builder.plugin(tauri_plugin_barcode_scanner::init());

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

            let state: AppState = Arc::new(Mutex::new(AppStateInner { data_dir, keypair }));
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
            discover_and_update,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска Tauri");
}
