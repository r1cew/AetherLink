use aetherlink_common::crypto::CryptoKeyPair;
use std::{fs, path::PathBuf};

// Псевдоним типа для сохранения обратной совместимости в коде телефона
pub type PhoneKeypair = CryptoKeyPair;

pub fn load_or_create(data_dir: &PathBuf) -> Result<PhoneKeypair, String> {
    let path = data_dir.join("phone_keypair.json");

    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&text).map_err(|e| e.to_string());
    }

    // Вызываем общий генератор ключей Curve25519
    let keypair = CryptoKeyPair::generate()?;

    let json = serde_json::to_string_pretty(&keypair).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(keypair)
}
