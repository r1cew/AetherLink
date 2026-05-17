/// Общее состояние приложения, доступное из Tauri-команд и TCP-сервера.
use std::{path::PathBuf, sync::Arc, time::Instant};

use tokio::sync::Mutex;

use crate::auth::{DeviceRegistry, ServerKeys};

// ─── Состояние паринга ────────────────────────────────────────────────────────

pub struct PairingSession {
    /// One-time token из QR-кода.
    pub token: String,
    /// Истекает через 120 секунд после генерации.
    pub expires_at: Instant,
}

impl PairingSession {
    pub fn is_valid(&self, token: &str) -> bool {
        self.token == token && Instant::now() < self.expires_at
    }
}

// ─── Основное состояние ───────────────────────────────────────────────────────

pub struct AppStateInner {
    /// Директория данных приложения (devices.json, server_keys.json, profiles.json).
    pub data_dir: PathBuf,

    /// Постоянная Noise keypair сервера.
    pub server_keys: ServerKeys,

    /// Реестр доверенных телефонов.
    pub registry: DeviceRegistry,

    /// Активная сессия паринга (есть пока показан QR).
    pub pairing: Option<PairingSession>,

    /// Developer Mode включён глобально (дополнительный рубильник безопасности).
    pub developer_mode_enabled: bool,

    /// App handle для отправки событий во фронтенд.
    pub app: tauri::AppHandle,
}

/// Arc<Mutex<...>> — шарится между Tauri-командами и tokio-задачами сервера.
pub type AppState = Arc<Mutex<AppStateInner>>;
