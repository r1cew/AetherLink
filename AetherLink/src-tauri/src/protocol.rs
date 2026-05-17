/// Все сообщения между телефоном (клиент) и ПК (сервер).
///
/// Телефон шлёт ClientRequest, ПК отвечает ServerResponse.
/// Всё завёрнуто в Noise-шифрование, так что JSON ходит в зашифрованном виде.
use serde::{Deserialize, Serialize};

// ─── Запросы от телефона ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientRequest {
    /// Первичная привязка устройства.
    /// Телефон присылает one-time token из QR и своё имя.
    Pair { token: String, name: String },

    /// Safe Mode — готовые системные команды.
    Safe {
        command: SafeCommand,
        #[serde(default)]
        params: serde_json::Value,
    },

    /// Automation Mode — запуск заранее созданного профиля.
    RunProfile { profile_id: String },

    /// Automation Mode — список всех профилей.
    ListProfiles,

    /// Developer Mode — raw команда в shell.
    Shell {
        cmd: String,
        #[serde(default = "ShellType::default")]
        shell: ShellType,
    },
}

// ─── Safe Mode команды ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeCommand {
    Shutdown,
    Sleep,
    Lock,
    VolumeUp,
    VolumeDown,
    VolumeSet, // params: { "level": 0-100 }
    MediaPlay,
    MediaPause,
    MediaNext,
    MediaPrev,
}

// ─── Shell type ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Cmd,
    Powershell,
}

impl ShellType {
    fn default() -> Self {
        ShellType::Powershell
    }
}

// ─── Ответ от сервера ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ServerResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ServerResponse {
    pub fn ok() -> Self {
        Self {
            ok: true,
            output: None,
            error: None,
            data: None,
        }
    }

    pub fn ok_output(output: impl Into<String>) -> Self {
        Self {
            ok: true,
            output: Some(output.into()),
            error: None,
            data: None,
        }
    }

    pub fn ok_data(data: serde_json::Value) -> Self {
        Self {
            ok: true,
            output: None,
            error: None,
            data: Some(data),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            output: None,
            error: Some(msg.into()),
            data: None,
        }
    }
}
