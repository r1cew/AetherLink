use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

// Тянем общие сущности из библиотеки
use aetherlink_common::crypto::{generate_pairing_token, CryptoKeyPair};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceMode {
    Default,
    Developer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    pub public_key_b64: String,
    pub mode: DeviceMode,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceRegistry {
    pub devices: Vec<TrustedDevice>,
}

impl DeviceRegistry {
    pub fn find_by_pubkey(&self, pubkey: &[u8]) -> Option<&TrustedDevice> {
        let key_b64 = B64.encode(pubkey);
        self.devices.iter().find(|d| d.public_key_b64 == key_b64)
    }

    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut TrustedDevice> {
        self.devices.iter_mut().find(|d| d.id == id)
    }

    pub fn add(&mut self, name: String, pubkey: &[u8]) -> TrustedDevice {
        let device = TrustedDevice {
            id: Uuid::new_v4().to_string(),
            name,
            public_key_b64: B64.encode(pubkey),
            mode: DeviceMode::Default,
        };
        self.devices.push(device.clone());
        device
    }
}

pub fn load_registry(dir: &PathBuf) -> Result<DeviceRegistry, String> {
    let path = dir.join("devices.json");
    if !path.exists() {
        return Ok(DeviceRegistry::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if text.trim().is_empty() {
        return Ok(DeviceRegistry::default());
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_registry(dir: &PathBuf, registry: &DeviceRegistry) -> Result<(), String> {
    let path = dir.join("devices.json");
    let json = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

// Псевдоним типа, чтобы тебе не пришлось переписывать типы во всем коде ПК!
pub type ServerKeys = CryptoKeyPair;

pub fn load_or_create_server_keys(dir: &PathBuf) -> Result<ServerKeys, String> {
    let path = dir.join("server_keys.json");

    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&text).map_err(|e| e.to_string());
    }

    // Вызываем генерацию из общей либы
    let keys = CryptoKeyPair::generate()?;

    let json = serde_json::to_string_pretty(&keys).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(keys)
}

// Оставляем обёртку старого метода, чтобы ничего не упало в других файлах ПК
pub fn new_pairing_token() -> String {
    generate_pairing_token()
}
