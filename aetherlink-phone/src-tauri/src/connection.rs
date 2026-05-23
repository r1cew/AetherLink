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

        let socket_addr: SocketAddr = addr.parse().map_err(|e| format!("Неверный адрес: {e}"))?;
        let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(4))
            .map_err(|e| format!("Превышен таймаут подключения к {addr}: {e}"))?;

        // КРИТИЧНО ДЛЯ ПИНГА: Отправляем пакеты мгновенно, не ждем буферизации ОС
        stream.set_nodelay(true).map_err(|e| e.to_string())?;

        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(15)))
            .map_err(|e| e.to_string())?;

        let mut buf = vec![0u8; 65540]; // 65535 + 4 байта под префикс длины
        let mut tmp = vec![0u8; 65540];

        let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
            .parse()
            .map_err(|_| "Неверные параметры Noise")?;

        let mut hs = Builder::new(params)
            .local_private_key(&keypair.private_key())
            .build_initiator()
            .map_err(|e: snow::Error| e.to_string())?;

        // msg1: → ПК (наш ephemeral key)
        let len = hs
            .write_message(&[], &mut buf[4..])
            .map_err(|e| e.to_string())?;
        buf[..4].copy_from_slice(&(len as u32).to_be_bytes());
        stream
            .write_all(&buf[..4 + len])
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        // msg2: ← ПК (его ephemeral + static keys)
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

        // Защита от MITM: сверяем публичный ключ ПК
        let remote_pubkey = hs
            .get_remote_static()
            .ok_or("Нет static key сервера после msg2")?;
        if B64.encode(remote_pubkey) != server.server_public_key_b64 {
            return Err("⚠️ Public key сервера не совпадает! Возможна MITM-атака.".into());
        }

        // msg3: → ПК (наш static key)
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

    /// Шаг 2. Отправка запроса по УЖЕ ОТКРЫТОМУ каналу (Работает мгновенно!)
    pub fn send_request(&mut self, request: &ClientRequest) -> Result<ServerResponse, String> {
        // Сериализуем прямо во внутренний буфер tmp, избегая аллокаций памяти
        let mut writer = std::io::Cursor::new(&mut self.tmp[..]);
        serde_json::to_writer(&mut writer, request).map_err(|e| e.to_string())?;
        let plain_len = writer.position() as usize;

        // Шифруем из tmp в buf со смещением в 4 байта (место под длину)
        let enc_len = self
            .transport
            .write_message(&self.tmp[..plain_len], &mut self.buf[4..])
            .map_err(|e: snow::Error| e.to_string())?;

        // Записываем длину заголовка
        let len_bytes = (enc_len as u32).to_be_bytes();
        self.buf[..4].copy_from_slice(&len_bytes);

        // Отправляем ВСЁ ОДНИМ системным вызовом со встроенным flush
        self.stream
            .write_all(&self.buf[..4 + enc_len])
            .map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;

        // Получаем ответ от ПК в наш buf
        let mut len_buf = [0u8; 4];
        self.stream
            .read_exact(&mut len_buf)
            .map_err(|e| e.to_string())?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        if resp_len > 65535 {
            return Err(format!("Слишком большой ответ: {resp_len}"));
        }

        self.stream
            .read_exact(&mut self.buf[..resp_len])
            .map_err(|e| e.to_string())?;

        // Дешифруем из buf в tmp
        let dec_len = self
            .transport
            .read_message(&self.buf[..resp_len], &mut self.tmp)
            .map_err(|e: snow::Error| e.to_string())?;

        // Возвращаем распарсенный JSON
        serde_json::from_slice(&self.tmp[..dec_len]).map_err(|e| e.to_string())
    }
}
