/// Automation Mode — профили с пользовательскими командами.
///
/// Профили создаются через UI ПК и хранятся в profiles.json.
/// Телефон может только запускать профиль по ID — никакого произвольного кода.
///
/// Пример запроса от телефона:
///   { "action": "run_profile", "profile_id": "start_mc_server" }
use std::{fs, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::ServerResponse;

// ─── Типы профилей ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProfileKind {
    /// Запустить .bat / .cmd файл.
    RunBat { path: String },
    /// Запустить .exe с аргументами.
    RunExe { path: String, args: Vec<String> },
    /// Выполнить PowerShell-скрипт (скрипт хранится на ПК, не принимается от телефона).
    PowerShell { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub kind: ProfileKind,
}

impl Profile {
    pub fn new(name: String, description: Option<String>, kind: ProfileKind) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            kind,
        }
    }
}

// ─── CRUD профилей ────────────────────────────────────────────────────────────

pub fn load(data_dir: &PathBuf) -> Result<Vec<Profile>, String> {
    let path = data_dir.join("profiles.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save(data_dir: &PathBuf, profiles: &[Profile]) -> Result<(), String> {
    let path = data_dir.join("profiles.json");
    let json = serde_json::to_string_pretty(profiles).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

// ─── Запуск профиля ───────────────────────────────────────────────────────────

pub fn run(data_dir: &PathBuf, profile_id: &str) -> ServerResponse {
    let profiles = match load(data_dir) {
        Ok(p) => p,
        Err(e) => return ServerResponse::err(format!("Не удалось загрузить профили: {e}")),
    };

    let Some(profile) = profiles.iter().find(|p| p.id == profile_id) else {
        return ServerResponse::err(format!("Профиль '{profile_id}' не найден"));
    };

    match execute(profile) {
        Ok(msg) => ServerResponse::ok_output(msg),
        Err(e) => ServerResponse::err(e),
    }
}

fn execute(profile: &Profile) -> Result<String, String> {
    match &profile.kind {
        ProfileKind::RunBat { path } => {
            Command::new("cmd")
                .args(["/c", path])
                .spawn()
                .map_err(|e| format!("Ошибка запуска bat: {e}"))?;
            Ok(format!("Запущено: {}", profile.name))
        }

        ProfileKind::RunExe { path, args } => {
            Command::new(path)
                .args(args)
                .spawn()
                .map_err(|e| format!("Ошибка запуска exe: {e}"))?;
            Ok(format!("Запущено: {}", profile.name))
        }

        ProfileKind::PowerShell { script } => {
            Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .spawn()
                .map_err(|e| format!("Ошибка PowerShell: {e}"))?;
            Ok(format!("Выполнено: {}", profile.name))
        }
    }
}
