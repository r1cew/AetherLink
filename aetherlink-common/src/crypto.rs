use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use snow::{Builder, HandshakeState, TransportState, params::NoiseParams};

/// Точная строка протокола Noise, используемая в AetherLink
pub const NOISE_PATTERN: &str = "Noise_XX_25519_AESGCM_SHA256";

/// Универсальная пара ключей для Noise (используется и ПК, и телефоном)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoKeyPair {
    pub private_key_b64: String,
    pub public_key_b64: String,
}

impl CryptoKeyPair {
    /// Декодировать приватный ключ из Base64 в байты
    pub fn private_key(&self) -> Vec<u8> {
        B64.decode(&self.private_key_b64)
            .expect("invalid private key")
    }

    /// Декодировать публичный ключ из Base64 в байты
    pub fn public_key(&self) -> Vec<u8> {
        B64.decode(&self.public_key_b64)
            .expect("invalid public key")
    }

    /// Генерирует новую пару ключей Curve25519 для протокола Noise
    pub fn generate() -> Result<Self, String> {
        let params: NoiseParams = NOISE_PATTERN
            .parse()
            .map_err(|_| "Неверные Noise параметры".to_string())?;

        let kp = Builder::new(params)
            .generate_keypair()
            .map_err(|e: snow::Error| e.to_string())?;

        Ok(Self {
            private_key_b64: B64.encode(&kp.private),
            public_key_b64: B64.encode(&kp.public),
        })
    }
}

/// Генерирует случайный одноразовый токен для QR-кода (32 байта, base64)
pub fn generate_pairing_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    B64.encode(bytes)
}

// ─── Хелперы для Хендшейка и Шифрования Фреймов ────────────────────────────────

pub fn init_initiator(local_key: &CryptoKeyPair) -> Result<HandshakeState, String> {
    let params: NoiseParams = NOISE_PATTERN.parse().map_err(|e| format!("{e:?}"))?;
    Builder::new(params)
        .local_private_key(&local_key.private_key())
        .build_initiator()
        .map_err(|e| e.to_string())
}

pub fn init_responder(local_key: &CryptoKeyPair) -> Result<HandshakeState, String> {
    let params: NoiseParams = NOISE_PATTERN.parse().map_err(|e| format!("{e:?}"))?;
    Builder::new(params)
        .local_private_key(&local_key.private_key())
        .build_responder()
        .map_err(|e| e.to_string())
}

pub fn encrypt_frame(transport: &mut TransportState, payload: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; payload.len() + 16]; // 16 байт под тег аутентификации AES-GCM
    let len = transport
        .write_message(payload, &mut buf)
        .map_err(|e| format!("Ошибка шифрования: {e}"))?;
    buf.truncate(len);
    Ok(buf)
}

pub fn decrypt_frame(
    transport: &mut TransportState,
    encrypted_payload: &[u8],
) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; encrypted_payload.len()];
    let len = transport
        .read_message(encrypted_payload, &mut buf)
        .map_err(|e| format!("Ошибка расшифрования: {e}"))?;
    buf.truncate(len);
    Ok(buf)
}
