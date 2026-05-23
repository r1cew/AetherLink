use std::{net::UdpSocket, time::Duration};

// Глобальные константы, общие для ПК и телефона
pub const BEACON_PORT: u16 = 9999;
pub const BEACON_MSG: &[u8] = b"AETHERLINK_8080";

// ─── СЕРВЕРНАЯ СТОРОНА (ПК) ──────────────────────────────────────────────────

/// Запускает бесконечный цикл — рассылает beacon каждые 2 секунды.
/// Использует асинхронный sleep из tokio.
pub async fn run_server_beacon() {
    loop {
        if let Err(e) = broadcast() {
            eprintln!("[beacon] broadcast error: {e}");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

fn broadcast() -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    socket
        .send_to(BEACON_MSG, format!("255.255.255.255:{BEACON_PORT}"))
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── КЛИЕНТСКАЯ СТОРОНА (Телефон) ────────────────────────────────────────────

/// Слушает beacon до таймаута. Возвращает IP-адрес найденного ПК.
/// Работает в синхронном (блокирующем) режиме.
pub fn discover_client(timeout_secs: u64) -> Result<String, String> {
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
