/// Keypair телефона для Noise XX.
/// Генерируется один раз при первом запуске, хранится в app data.
/// Это "удостоверение личности" телефона — по нему ПК узнаёт устройство.
use std::{fs, path::PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use snow::{params::NoiseParams, Builder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneKeypair {
    pub private_key_b64: String,
    pub public_key_b64: String,
}

impl PhoneKeypair {
    pub fn private_key(&self) -> Vec<u8> {
        B64.decode(&self.private_key_b64)
            .expect("invalid private key")
    }
}

pub fn load_or_create(data_dir: &PathBuf) -> Result<PhoneKeypair, String> {
    let path = data_dir.join("phone_keypair.json");

    if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return serde_json::from_str(&text).map_err(|e| e.to_string());
    }

    let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
        .parse()
        .map_err(|_| "invalid noise params")?;

    let kp = Builder::new(params)
        .generate_keypair()
        .map_err(|e: snow::Error| e.to_string())?;

    let keypair = PhoneKeypair {
        private_key_b64: B64.encode(&kp.private),
        public_key_b64: B64.encode(&kp.public),
    };

    let json = serde_json::to_string_pretty(&keypair).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;

    Ok(keypair)
}
