use serde::{Deserialize, Serialize};

// ─── Запросы от телефона ──────────────────────────────────────────────────────

// Добавляем и Serialize, и Deserialize, и Clone, чтобы обе стороны могли работать с enum
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientRequest {
    /// Первичная привязка устройства.
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Cmd,
    Powershell,
}

impl ShellType {
    // Дефолт нужен для Serde, оставляем его публичным
    pub fn default() -> Self {
        ShellType::Powershell
    }
}

// ─── Ответ от сервера ─────────────────────────────────────────────────────────

// Объединяем: даем и Serialize (для ПК), и Deserialize (для телефона)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerResponse {
    pub ok: bool,
    // skip_serializing_if экономит трафик в сети, оставляя JSON чистым
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
}

// Конструкторы ответов (теперь доступны и на ПК для отправки, и на мобилке для тестов)
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
