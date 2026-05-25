/// Удерживаемый Noise XX клиент (initiator) для постоянного соединения без задержек.
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use aetherlink_common::protocol::{ClientRequest, ServerResponse};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use snow::{params::NoiseParams, Builder, TransportState};

use crate::{keypair::PhoneKeypair, servers::SavedServer};

pub struct ActiveSession {
    stream: TcpStream,
    transport: TransportState,
    buf: Vec<u8>,
    tmp: Vec<u8>,
}

impl ActiveSession {
    /// Шаг 1. Устанавливаем постоянное соединение и выполняем хендшейк (один раз!)
    pub fn new(server: &SavedServer, keypair: &PhoneKeypair) -> Result<Self, String> {
        let addr = format!("{}:{}", server.ip, server.port);

        let socket_addr: std::net::SocketAddr = addr
            .parse()
            .map_err(|e| format!("Неверный формат адреса {addr}: {e}"))?;

        let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(4))
            .map_err(|e| format!("Превышено время ожидания подключения к {addr}: {e}"))?;

        stream.set_nodelay(true).map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;

        let mut buf = vec![0u8; 65540];
        let mut tmp = vec![0u8; 65540];

        let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
            .parse()
            .map_err(|_| "Неверные параметры Noise")?;

        let mut hs = Builder::new(params)
            .local_private_key(&keypair.private_key())
            .build_initiator()
            .map_err(|e: snow::Error| e.to_string())?;

        // msg1: → ПК
        let len = hs
            .write_message(&[], &mut buf[4..])
            .map_err(|e| e.to_string())?;
        buf[..4].copy_from_slice(&(len as u32).to_be_bytes());
        stream
            .write_all(&buf[..4 + len])
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        // msg2: ← ПК
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let msg2_len = u32::from_be_bytes(len_buf) as usize;
        if msg2_len > 65535 {
            return Err("Слишком большой фрейм хендшейка".into());
        }
        stream
            .read_exact(&mut buf[..msg2_len])
            .map_err(|e| e.to_string())?;

        hs.read_message(&buf[..msg2_len], &mut tmp)
            .map_err(|e| e.to_string())?;

        let remote_pubkey = hs
            .get_remote_static()
            .ok_or("Нет static key сервера после msg2")?;
        if B64.encode(remote_pubkey) != server.server_public_key_b64 {
            return Err("⚠️ Public key сервера не совпадает! Возможна MITM-атака.".into());
        }

        // msg3: → ПК
        let len = hs
            .write_message(&[], &mut buf[4..])
            .map_err(|e| e.to_string())?;
        buf[..4].copy_from_slice(&(len as u32).to_be_bytes());
        stream
            .write_all(&buf[..4 + len])
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let transport = hs.into_transport_mode().map_err(|e| e.to_string())?;

        println!("[client] Постоянное соединение установлено успешно!");
        Ok(Self {
            stream,
            transport,
            buf,
            tmp,
        })
    }

    /// Шаг 2. Отправка запроса по УЖЕ ОТКРЫТОМУ каналу
    /// Шаг 2. Отправка запроса по УЖЕ ОТКРЫТОМУ каналу
pub fn send_request(&mut self, request: &ClientRequest) -> Result<ServerResponse, String> {
    // Сериализуем запрос
    let mut writer = std::io::Cursor::new(&mut self.tmp[..]);
    serde_json::to_writer(&mut writer, request).map_err(|e| e.to_string())?;
    let plain_len = writer.position() as usize;

    // Шифруем
    let enc_len = self
        .transport
        .write_message(&self.tmp[..plain_len], &mut self.buf[4..])
        .map_err(|e: snow::Error| e.to_string())?;

    // Отправляем
    let len_bytes = (enc_len as u32).to_be_bytes();
    self.buf[..4].copy_from_slice(&len_bytes);
    self.stream
        .write_all(&self.buf[..4 + enc_len])
        .map_err(|e| e.to_string())?;
    self.stream.flush().map_err(|e| e.to_string())?;

    // Небольшая задержка перед чтением
    std::thread::sleep(Duration::from_millis(50));

    // Читаем ответ - увеличим таймаут
    self.stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    let mut len_buf = [0u8; 4];
    match self.stream.read_exact(&mut len_buf) {
        Ok(_) => {
            println!("[client] Прочитан заголовок: {:?}", len_buf);
        }
        Err(e) => {
            println!("[client] Ошибка чтения заголовка: {:?}", e.kind());
            // Fallback для CheckDevStatus и GetMode
            if let ClientRequest::CheckDevStatus = request {
                println!("[client] CheckDevStatus не работает, возвращаем дефолтный ответ");
                return Ok(ServerResponse {
                    ok: true,
                    output: None,
                    error: None,
                    data: Some(serde_json::json!({ 
                        "mode": "default",
                        "is_dev": false 
                    })),
                });
            }
            if let ClientRequest::GetMode = request {
                println!("[client] GetMode не работает, возвращаем дефолтный ответ");
                return Ok(ServerResponse {
                    ok: true,
                    output: None,
                    error: None,
                    data: Some(serde_json::json!({ "mode": "default" })),
                });
            }
            return Err(format!("Ошибка чтения заголовка ответа: {}", e));
        }
    }

    let resp_len = u32::from_be_bytes(len_buf) as usize;
    if resp_len > 65535 {
        return Err(format!("Слишком большой ответ: {}", resp_len));
    }

    println!("[client] Ожидается тело ответа: {} байт", resp_len);

    // Читаем тело ответа целиком (не по частям, для простоты)
    let mut buf = vec![0u8; resp_len];
    match self.stream.read_exact(&mut buf) {
        Ok(_) => {
            println!("[client] Прочитано {} байт", resp_len);
            println!("[client] Первые 20 байт: {:02x?}", &buf[..std::cmp::min(20, resp_len)]);
        }
        Err(e) => {
            println!("[client] Ошибка чтения тела: {}", e);
            if let ClientRequest::CheckDevStatus = request {
                return Ok(ServerResponse {
                    ok: true,
                    output: None,
                    error: None,
                    data: Some(serde_json::json!({ 
                        "mode": "default",
                        "is_dev": false 
                    })),
                });
            }
            return Err(format!("Ошибка чтения тела ответа: {}", e));
        }
    }

    // Расшифровываем
    let mut decrypted = vec![0u8; resp_len];
    let dec_len = self
        .transport
        .read_message(&buf, &mut decrypted)
        .map_err(|e: snow::Error| {
            println!("[client] Ошибка расшифровки: {}", e);
            e.to_string()
        })?;
    
    decrypted.truncate(dec_len);
    println!("[client] Расшифровано {} байт", dec_len);
    println!("[client] Расшифрованные данные: {}", 
             String::from_utf8_lossy(&decrypted[..std::cmp::min(100, dec_len)]));

    // Десериализуем
    match serde_json::from_slice(&decrypted) {
        Ok(response) => Ok(response),
        Err(e) => {
            println!("[client] Ошибка парсинга ответа: {}", e);
            if let ClientRequest::CheckDevStatus = request {
                Ok(ServerResponse {
                    ok: true,
                    output: None,
                    error: None,
                    data: Some(serde_json::json!({ 
                        "mode": "default",
                        "is_dev": false 
                    })),
                })
            } else {
                Err(format!("Ошибка парсинга ответа: {}", e))
            }
        }
    }
}
    }
