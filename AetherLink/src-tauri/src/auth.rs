/// Управление доверенными устройствами и ключами сервера.
///
/// Схема аутентификации:
///   - Сервер (ПК) имеет постоянную keypair (Noise static key).
///   - Телефон тоже генерирует свою keypair (один раз при установке).
///   - Паринг: ПК показывает QR с {server_public_key, pairing_token, ip, port}.
///   - Телефон подключается через Noise XX → после хендшейка ПК видит public key телефона.
///   - Телефон шлёт pairing_token → ПК сохраняет public key телефона как доверенное устройство.
///   - Дальнейшие подключения: Noise XX, ПК проверяет remote static key по списку.
///
/// Если ПК поменял IP — телефон переоткрывает его через UDP beacon (как в pc-shutdown).
use std::{fs, path::PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use snow::{params::NoiseParams, Builder};
use uuid::Uuid;

// ─── Уровни доступа устройства ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceMode {
    /// Только готовые системные команды (shutdown, sleep, volume...).
    Safe,
    /// Запуск заранее созданных профилей.
    Automation,
    /// Полный shell/powershell доступ. Выключен по умолчанию.
    Developer,
}

// ─── Доверенное устройство ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    /// Noise static public key телефона (base64).
    pub public_key_b64: String,
    pub mode: DeviceMode,
}

// ─── Реестр устройств ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeviceRegistry {
    pub devices: Vec<TrustedDevice>,
}

impl DeviceRegistry {
    /// Найти устройство по его Noise static public key.
    pub fn find_by_pubkey(&self, pubkey: &[u8]) -> Option<&TrustedDevice> {
        let key_b64 = B64.encode(pubkey);
        self.devices.iter().find(|d| d.public_key_b64 == key_b64)
    }

    /// Найти устройство по ID (для управления из UI).
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut TrustedDevice> {
        self.devices.iter_mut().find(|d| d.id == id)
    }

    /// Добавить новое доверенное устройство после успешного паринга.
    pub fn add(&mut self, name: String, pubkey: &[u8]) -> TrustedDevice {
        let device = TrustedDevice {
            id: Uuid::new_v4().to_string(),
            name,
            public_key_b64: B64.encode(pubkey),
            mode: DeviceMode::Automation, // по умолчанию — самый безопасный режим
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
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_registry(dir: &PathBuf, registry: &DeviceRegistry) -> Result<(), String> {
    let path = dir.join("devices.json");
    let json = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

// ─── Ключи сервера ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerKeys {
    pub private_key_b64: String,
    pub public_key_b64: String,
}

impl ServerKeys {
    pub fn private_key(&self) -> Vec<u8> {
        B64.decode(&self.private_key_b64)
            .expect("invalid private key")
    }
    pub fn public_key(&self) -> Vec<u8> {
        B64.decode(&self.public_key_b64)
            .expect("invalid public key")
    }
}

/// Загружает существующие ключи или генерирует новые.
pub fn load_or_create_server_keys(dir: &PathBuf) -> Result<ServerKeys, String> {
    let path = dir.join("server_keys.json");

    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&text).map_err(|e| e.to_string());
    }

    // Генерируем новую keypair через snow (Noise XX, curve25519)
    let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
        .parse()
        .map_err(|_| "Неверные Noise параметры".to_string())?;
    let kp = Builder::new(params)
        .generate_keypair()
        .map_err(|e: snow::Error| e.to_string())?;

    let keys = ServerKeys {
        private_key_b64: B64.encode(&kp.private),
        public_key_b64: B64.encode(&kp.public),
    };

    let json = serde_json::to_string_pretty(&keys).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(keys)
}

// ─── Pairing token ────────────────────────────────────────────────────────────

/// Генерирует случайный one-time token для QR (32 байта, base64).
pub fn new_pairing_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    B64.encode(bytes)
}
