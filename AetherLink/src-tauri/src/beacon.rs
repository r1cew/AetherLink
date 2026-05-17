/// UDP-beacon — широковещательный сигнал ПК в локальной сети.
///
/// Телефон слушает на порту BEACON_PORT.
/// Если ПК поменял IP — телефон переоткрывает его через этот маяк (как в pc-shutdown).
use std::{net::UdpSocket, time::Duration};

pub const BEACON_PORT: u16 = 9999;
/// Сообщение beacon. Телефон ищет именно этот байт-паттерн.
pub const BEACON_MSG: &[u8] = b"AETHERLINK_8080";

/// Запускает бесконечный цикл — рассылает beacon каждые 3 секунды.
pub async fn run() {
    loop {
        if let Err(e) = broadcast() {
            eprintln!("[beacon] broadcast error: {e}");
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
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
