/// Клиентская сторона beacon — слушаем UDP-broadcast от ПК.
///
/// Если IP ПК сменился, телефон всё равно найдёт его по beacon
/// (ПК шлёт "AETHERLINK_8080" каждые 3 сек на порт 9999).
use std::{net::UdpSocket, time::Duration};

pub const BEACON_PORT: u16  = 9999;
pub const BEACON_MSG: &[u8] = b"AETHERLINK_8080";

/// Слушает beacon до таймаута. Возвращает IP-адрес найденного ПК.
pub fn discover(timeout_secs: u64) -> Result<String, String> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{BEACON_PORT}"))
        .map_err(|e| format!("Не удалось занять порт {BEACON_PORT}: {e}"))?;

    socket
        .set_read_timeout(Some(Duration::from_secs(timeout_secs)))
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 256];
    loop {
        let (size, addr) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
        if &buf[..size] == BEACON_MSG {
            return Ok(addr.ip().to_string());
        }
    }
}
