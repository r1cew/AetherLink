/// Noise XX клиент (initiator).
///
/// Поток работы:
///   1. TCP connect к IP:port сервера.
///   2. Noise XX хендшейк (3 сообщения).
///   3. Проверяем server_public_key из сохранённых данных — защита от MITM.
///   4. Шифруем и отправляем ClientRequest.
///   5. Дешифруем ServerResponse.
///
/// Если connect падает → вызывающий код пробует beacon для обнаружения нового IP.
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use aetherlink_common::protocol::{ClientRequest, ServerResponse};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use snow::{params::NoiseParams, Builder, TransportState};

use crate::{keypair::PhoneKeypair, servers::SavedServer};

/// Подключиться к серверу, отправить запрос, вернуть ответ.
pub fn send(
    server: &SavedServer,
    keypair: &PhoneKeypair,
    request: &ClientRequest,
) -> Result<ServerResponse, String> {
    let addr = format!("{}:{}", server.ip, server.port);

    let mut stream =
        TcpStream::connect(&addr).map_err(|e| format!("Нет соединения с {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;

    let mut transport = handshake(&mut stream, keypair, &server.server_public_key_b64)?;

    // Шифруем и отправляем запрос
    let req_json = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 65535];
    let enc_len = transport
        .write_message(&req_json, &mut buf)
        .map_err(|e: snow::Error| e.to_string())?;
    write_frame(&mut stream, &buf[..enc_len])?;

    // Получаем и дешифруем ответ
    let encrypted = read_frame(&mut stream)?;
    let dec_len = transport
        .read_message(&encrypted, &mut buf)
        .map_err(|e: snow::Error| e.to_string())?;

    serde_json::from_slice(&buf[..dec_len]).map_err(|e| e.to_string())
}

// ─── Noise XX хендшейк (инициатор) ───────────────────────────────────────────

fn handshake(
    stream: &mut TcpStream,
    keypair: &PhoneKeypair,
    expected_pubkey_b64: &str,
) -> Result<TransportState, String> {
    let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
        .parse()
        .map_err(|_| "invalid noise params")?;

    let private_key = keypair.private_key();

    let mut hs = Builder::new(params)
        .local_private_key(&private_key)
        .build_initiator()
        .map_err(|e: snow::Error| e.to_string())?;

    let mut buf = vec![0u8; 65535];
    let mut tmp = vec![0u8; 65535];

    // msg1: → ПК (наш ephemeral key)
    let len = hs
        .write_message(&[], &mut buf)
        .map_err(|e: snow::Error| e.to_string())?;
    write_frame(stream, &buf[..len])?;

    // msg2: ← ПК (его ephemeral + static keys)
    let msg2 = read_frame(stream)?;
    hs.read_message(&msg2, &mut tmp)
        .map_err(|e: snow::Error| e.to_string())?;

    // ── Проверка public key ПК — защита от MITM ──────────────────────────────
    // Телефон при паринге сохранил server_public_key из QR.
    // Сейчас сравниваем с тем, что пришло в хендшейке.
    let remote_pubkey = hs
        .get_remote_static()
        .ok_or("Нет static key сервера после msg2")?;
    let remote_b64 = B64.encode(remote_pubkey);
    if remote_b64 != expected_pubkey_b64 {
        return Err("⚠️ Public key сервера не совпадает! Возможна MITM-атака.".into());
    }

    // msg3: → ПК (наш static key)
    let len = hs
        .write_message(&[], &mut buf)
        .map_err(|e: snow::Error| e.to_string())?;
    write_frame(stream, &buf[..len])?;

    hs.into_transport_mode()
        .map_err(|e: snow::Error| e.to_string())
}

// ─── Фреймирование: [u32 BE: длина][данные] ──────────────────────────────────

fn write_frame(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
    stream
        .write_all(&(data.len() as u32).to_be_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(data).map_err(|e| e.to_string())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 65_000 {
        return Err(format!("Слишком большой фрейм: {len}"));
    }
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).map_err(|e| e.to_string())?;
    Ok(data)
}
