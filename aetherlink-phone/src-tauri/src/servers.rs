/// Список привязанных ПК.
/// Каждый сервер хранит IP (может меняться), порт и public key ПК (для MITM-проверки).
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedServer {
    pub id: String,
    pub name: String,
    /// Текущий IP — может быть обновлён через beacon при смене адреса.
    pub ip: String,
    pub port: u16,
    /// Noise static public key ПК (base64) — для защиты от MITM.
    pub server_public_key_b64: String,
}

pub fn load(data_dir: &PathBuf) -> Result<Vec<SavedServer>, String> {
    let path = data_dir.join("servers.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save(data_dir: &PathBuf, servers: &[SavedServer]) -> Result<(), String> {
    let path = data_dir.join("servers.json");
    let json = serde_json::to_string_pretty(servers).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}
