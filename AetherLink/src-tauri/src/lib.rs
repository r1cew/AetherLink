/// AetherLink — точка входа Tauri-приложения.
///
/// Tauri-команды (вызываются из Vue-фронтенда):
///
/// ┌────────────────────────────┬──────────────────────────────────────────────┐
/// │ Команда                    │ Назначение                                   │
/// ├────────────────────────────┼──────────────────────────────────────────────┤
/// │ generate_pairing_qr()      │ Генерирует данные для QR (JSON-строка)       │
/// │ get_devices()              │ Список привязанных устройств                 │
/// │ set_device_mode(id, mode)  │ Сменить режим устройства (safe/auto/dev)     │
/// │ remove_device(id)          │ Удалить устройство из реестра                │
/// │ set_developer_mode(bool)   │ Глобальный рубильник Developer Mode          │
/// │ get_profiles()             │ Список Automation-профилей                   │
/// │ create_profile(json)       │ Создать новый профиль                        │
/// │ delete_profile(id)         │ Удалить профиль                              │
/// └────────────────────────────┴──────────────────────────────────────────────┘
///
/// Tauri-события (шлются во фронтенд из Rust):
///   "device-paired"  →  { id, name, mode }   когда телефон успешно привязался
use std::{sync::Arc, time::Duration};

use tauri::Manager;
use tokio::sync::Mutex;

mod auth;
mod beacon;
mod modes;
mod protocol;
mod server;
mod state;

use auth::{
    load_or_create_server_keys, load_registry, new_pairing_token, save_registry, DeviceMode,
};
use modes::automation::{self, Profile, ProfileKind};
use state::{AppState, AppStateInner, PairingSession};

// ─── Вспомогательная: IP локальной машины ────────────────────────────────────

fn local_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| s.connect("8.8.8.8:80").ok().map(|_| s))
        .and_then(|s| s.local_addr().ok())
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

// ─── Tauri-команды ────────────────────────────────────────────────────────────

/// Генерирует QR-данные для паринга нового устройства.
///
/// Возвращает JSON-строку — друг отрисует её как QR на экране.
/// Токен действителен 120 секунд. После скана телефоном — стирается.
///
/// QR содержит:
///   { "ip", "port", "server_public_key", "pairing_token" }
///
/// Телефон должен:
///   1. Подключиться по IP:port через Noise XX.
///   2. Убедиться, что remote static key сервера == server_public_key из QR (защита от MITM).
///   3. Отправить { "action": "pair", "token": "...", "name": "Мой телефон" }.
#[tauri::command]
async fn generate_pairing_qr(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let token = new_pairing_token();
    let ip = local_ip();

    let server_public_key = {
        let mut s = state.inner().lock().await;
        s.pairing = Some(PairingSession {
            token: token.clone(),
            expires_at: std::time::Instant::now() + Duration::from_secs(120),
        });
        s.server_keys.public_key_b64.clone()
    };

    let qr = serde_json::json!({
        "ip":               ip,
        "port":             server::PORT,
        "server_public_key": server_public_key,
        "pairing_token":    token,
    });

    Ok(qr.to_string())
}

/// Список всех привязанных устройств (без private keys).
#[tauri::command]
async fn get_devices(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let s = state.inner().lock().await;
    let list: Vec<_> = s
        .registry
        .devices
        .iter()
        .map(|d| {
            serde_json::json!({
                "id":   d.id,
                "name": d.name,
                "mode": d.mode,
            })
        })
        .collect();
    Ok(serde_json::Value::Array(list))
}

/// Сменить режим устройства: "safe" | "automation" | "developer".
#[tauri::command]
async fn set_device_mode(
    state: tauri::State<'_, AppState>,
    device_id: String,
    mode: String,
) -> Result<(), String> {
    let new_mode = match mode.as_str() {
        "default" => DeviceMode::Automation,
        "developer" => DeviceMode::Developer,
        other => return Err(format!("Неизвестный режим: {other}")),
    };

    let mut s = state.inner().lock().await;
    s.registry
        .find_by_id_mut(&device_id)
        .ok_or_else(|| format!("Устройство {device_id} не найдено"))?
        .mode = new_mode;

    save_registry(&s.data_dir, &s.registry)
}

/// Удалить устройство из реестра (отозвать доступ).
#[tauri::command]
async fn remove_device(state: tauri::State<'_, AppState>, device_id: String) -> Result<(), String> {
    let mut s = state.inner().lock().await;
    s.registry.devices.retain(|d| d.id != device_id);
    save_registry(&s.data_dir, &s.registry)
}

/// Глобальный рубильник Developer Mode.
/// Даже если устройство имеет режим Developer — shell не работает без этого флага.
#[tauri::command]
async fn set_developer_mode(
    state: tauri::State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state.inner().lock().await.developer_mode_enabled = enabled;
    Ok(())
}

/// Список Automation-профилей (для отображения в UI ПК).
#[tauri::command]
async fn get_profiles(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let data_dir = state.inner().lock().await.data_dir.clone();

    let profiles = automation::load(&data_dir)?;
    serde_json::to_value(profiles).map_err(|e| e.to_string())
}

/// Создать новый профиль.
///
/// Пример вызова из Vue:
/// ```
/// await invoke('create_profile', {
///   profile: {
///     name: "Minecraft Server",
///     description: "Запускает сервер",
///     kind: { type: "run_bat", path: "C:\\servers\\mc\\start.bat" }
///   }
/// })
/// ```
#[tauri::command]
async fn create_profile(
    state: tauri::State<'_, AppState>,
    name: String,
    description: Option<String>,
    kind: serde_json::Value,
) -> Result<String, String> {
    let kind: ProfileKind = serde_json::from_value(kind).map_err(|e| e.to_string())?;
    let profile = Profile::new(name, description, kind);
    let id = profile.id.clone();

    let data_dir = state.inner().lock().await.data_dir.clone();
    let mut profiles = automation::load(&data_dir)?;
    profiles.push(profile);
    automation::save(&data_dir, &profiles)?;

    Ok(id)
}

/// Удалить профиль по ID.
#[tauri::command]
async fn delete_profile(
    state: tauri::State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    let data_dir = state.inner().lock().await.data_dir.clone();
    let mut profiles = automation::load(&data_dir)?;
    profiles.retain(|p| p.id != profile_id);
    automation::save(&data_dir, &profiles)
}

// ─── Точка входа приложения ───────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Не удалось получить app data dir");
            std::fs::create_dir_all(&data_dir).expect("Не удалось создать директорию");

            let server_keys =
                load_or_create_server_keys(&data_dir).expect("Ошибка загрузки ключей сервера");
            let registry = load_registry(&data_dir).expect("Ошибка загрузки реестра устройств");

            let state: AppState = Arc::new(Mutex::new(AppStateInner {
                data_dir,
                server_keys,
                registry,
                pairing: None,
                developer_mode_enabled: false,
                app: app.handle().clone(),
            }));

            app.manage(state.clone());

            // Запускаем TCP-сервер и UDP-beacon в фоне
            tauri::async_runtime::spawn(server::run(state.clone()));
            tauri::async_runtime::spawn(beacon::run());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_pairing_qr,
            get_devices,
            set_device_mode,
            remove_device,
            set_developer_mode,
            get_profiles,
            create_profile,
            delete_profile,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска Tauri");
}
