/// Типы сообщений — должны точно совпадать с сервером (AetherLink ПК).
/// Телефон сериализует ClientRequest → отправляет.
/// Телефон десериализует ServerResponse ← получает.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ClientRequest {
    Pair {
        token: String,
        name: String,
    },
    Safe {
        command: SafeCommand,
        #[serde(default)]
        params: serde_json::Value,
    },
    RunProfile {
        profile_id: String,
    },
    ListProfiles,
    Shell {
        cmd: String,
        shell: ShellType,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeCommand {
    Shutdown,
    Sleep,
    Lock,
    VolumeUp,
    VolumeDown,
    VolumeSet,
    MediaPlay,
    MediaPause,
    MediaNext,
    MediaPrev,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Cmd,
    Powershell,
}

#[derive(Debug, Deserialize)]
pub struct ServerResponse {
    pub ok: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}
