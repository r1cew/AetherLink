/// TCP-сервер с Noise XX шифрованием.
///
/// Протокол Noise XX (curve25519, AESGCM, SHA256):
///   - Обе стороны обмениваются static keys в процессе хендшейка.
///   - ПК не знает public key телефона заранее → при паринге сохраняет его.
///   - При последующих подключениях — ищет remote static key в реестре.
///
/// Фреймирование: каждое сообщение = [4 байта BE: длина][N байт: данные].
///
/// Паринг-токен из QR-кода действителен 120 секунд.
/// После успешного паринга — стирается (one-time use).
///
/// Событие "device-paired" эмитируется во фронтенд Tauri при успешном паринге.
use snow::{params::NoiseParams, Builder};
use tauri::Emitter;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    auth::{save_registry, DeviceMode, TrustedDevice},
    modes::{automation, profiles, safe, shell},
    protocol::{ClientRequest, ServerResponse},
    state::AppState,
};

pub const PORT: u16 = 8080;

// ─── Точка входа сервера ──────────────────────────────────────────────────────

pub async fn run(state: AppState) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{PORT}")).await {
        Ok(l) => {
            println!("[server] Слушаю на порту {PORT}");
            l
        }
        Err(e) => {
            eprintln!("[server] Ошибка bind: {e}");
            return;
        }
    };

    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[server] Accept error: {e}");
                continue;
            }
        };
        println!("[server] Подключение от {addr}");

        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle(stream, state).await {
                eprintln!("[server] Ошибка сессии {addr}: {e}");
            }
        });
    }
}

// ─── Обработка одного подключения ────────────────────────────────────────────

async fn handle(mut stream: TcpStream, state: AppState) -> Result<(), String> {
    stream
        .set_nodelay(true)
        .map_err(|e: std::io::Error| format!("nodelay: {e}"))?;

    // Берём private key сервера — нужен до хендшейка.
    let private_key = state.lock().await.server_keys.private_key();

    let params: NoiseParams = "Noise_XX_25519_AESGCM_SHA256"
        .parse()
        .map_err(|_| "Неверные Noise параметры".to_string())?;

    let mut handshake = Builder::new(params)
        .local_private_key(&private_key)
        .build_responder()
        .map_err(|e: snow::Error| e.to_string())?;

    // ── Noise XX хендшейк (3 сообщения) ──────────────────────────────────────
    //
    //  Инициатор (телефон) → Респондер (ПК):
    //    msg1:  e                   (телефон шлёт ephemeral key)
    //    msg2:  e, ee, s, es        (ПК шлёт ephemeral + static keys, DH)
    //    msg3:  s, se               (телефон шлёт static key, DH)
    //
    //  После msg3 оба знают static keys друг друга → transport mode.

    let mut buf = vec![0u8; 65535];
    let mut tmp = vec![0u8; 65535];

    // msg1: recv ← телефон
    let msg1 = recv_frame(&mut stream).await?;
    handshake
        .read_message(&msg1, &mut tmp)
        .map_err(|e| format!("handshake msg1: {e}"))?;

    // msg2: send → телефон
    let len = handshake
        .write_message(&[], &mut buf)
        .map_err(|e| format!("handshake msg2: {e}"))?;
    send_frame(&mut stream, &buf[..len]).await?;

    // msg3: recv ← телефон
    let msg3 = recv_frame(&mut stream).await?;
    handshake
        .read_message(&msg3, &mut tmp)
        .map_err(|e| format!("handshake msg3: {e}"))?;

    // Получаем remote static public key телефона (стал известен после msg3).
    let remote_pubkey = handshake
        .get_remote_static()
        .ok_or("Нет remote static key после хендшейка")?
        .to_vec();

    let mut transport = handshake.into_transport_mode().map_err(|e| e.to_string())?;

    println!("[server] Хендшейк завершён");

    // ── Основной цикл сообщений ───────────────────────────────────────────────

    loop {
        let encrypted = match recv_frame(&mut stream).await {
            Ok(d) => d,
            Err(_) => break, // клиент отключился
        };

        // Расшифровываем
        let plain_len = transport
            .read_message(&encrypted, &mut buf)
            .map_err(|e| format!("decrypt: {e}"))?;

        let request: ClientRequest =
            serde_json::from_slice(&buf[..plain_len]).map_err(|e| format!("json parse: {e}"))?;

        // Обрабатываем запрос
        let response = dispatch(request, &remote_pubkey, &state).await;

        // Шифруем и отправляем ответ
        let resp_json = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
        let enc_len = transport
            .write_message(&resp_json, &mut buf)
            .map_err(|e| format!("encrypt: {e}"))?;
        send_frame(&mut stream, &buf[..enc_len]).await?;
    }

    Ok(())
}

// ─── Роутинг запросов ─────────────────────────────────────────────────────────

async fn dispatch(
    request: ClientRequest,
    remote_pubkey: &[u8],
    state: &AppState,
) -> ServerResponse {
    match request {
        // Паринг обрабатывается всегда — независимо от того, есть ли устройство в реестре.
        ClientRequest::Pair { token, name } => pair(state, remote_pubkey, &token, &name).await,

        // Все остальные запросы — только для доверенных устройств.
        other => {
            let device = {
                let s = state.lock().await;
                s.registry.find_by_pubkey(remote_pubkey).cloned()
            };

            match device {
                None => ServerResponse::err("Устройство не привязано. Сначала выполните паринг."),
                Some(d) => route(other, &d, state).await,
            }
        }
    }
}

async fn route(request: ClientRequest, device: &TrustedDevice, state: &AppState) -> ServerResponse {
    let data_dir = state.lock().await.data_dir.clone();

    match request {
        // ── Default Mode: системные команды ───────────────────────────────────────────────
        ClientRequest::Safe { command, params } => safe::execute(command, params),

        // ── Default Mode: профили автоматизации ─────────────────────────────────────────
        ClientRequest::RunProfile { profile_id } => profiles::run_profile(&data_dir, &profile_id),

        ClientRequest::ListProfiles => match automation::load(&data_dir) {
            Ok(profiles) => {
                ServerResponse::ok_data(serde_json::to_value(&profiles).unwrap_or_default())
            }
            Err(e) => ServerResponse::err(e),
        },

        // ── Developer Mode ─────────────────────────────────────────────────────
        ClientRequest::Shell { cmd, shell } => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для Shell нужен режим Developer.");
            }
            shell::execute(cmd, shell)
        }

        // Паринг уже обработан выше — сюда не дойдёт.
        ClientRequest::Pair { .. } => ServerResponse::err("Устройство уже привязано."),
    }
}

// ─── Паринг ───────────────────────────────────────────────────────────────────

async fn pair(state: &AppState, remote_pubkey: &[u8], token: &str, name: &str) -> ServerResponse {
    let mut s = state.lock().await;

    // Проверяем одноразовый токен
    let valid = s
        .pairing
        .as_ref()
        .map(|p| p.is_valid(token))
        .unwrap_or(false);

    if !valid {
        return ServerResponse::err("Токен паринга недействителен или истёк.");
    }

    // Проверяем, не привязано ли уже это устройство
    if s.registry.find_by_pubkey(remote_pubkey).is_some() {
        return ServerResponse::err("Устройство уже привязано.");
    }

    // Добавляем в реестр
    let device = s.registry.add(name.to_string(), remote_pubkey);
    let device_id = device.id.clone();

    // Сохраняем реестр на диск
    if let Err(e) = save_registry(&s.data_dir, &s.registry) {
        return ServerResponse::err(format!("Ошибка сохранения реестра: {e}"));
    }

    // Токен использован — стираем (one-time use!)
    s.pairing = None;

    // Уведомляем фронтенд Tauri — UI может обновить список устройств
    let _ = s.app.emit(
        "device-paired",
        serde_json::json!({
            "id":   device_id,
            "name": name,
            "mode": "default",
        }),
    );

    println!("[server] Новое устройство привязано: '{name}' (id={device_id})");

    ServerResponse::ok_data(serde_json::json!({
        "device_id": device_id,
        "name": name,
        "mode": "default",
    }))
}

// ─── Фреймирование ────────────────────────────────────────────────────────────
// Каждое сообщение: [u32 BE: длина][данные]

async fn recv_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| e.to_string())?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 65_000 {
        return Err(format!("Слишком большой фрейм: {len} байт"));
    }

    let mut data = vec![0u8; len];
    stream
        .read_exact(&mut data)
        .await
        .map_err(|e| e.to_string())?;

    Ok(data)
}

async fn send_frame(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len).await.map_err(|e| e.to_string())?;
    stream.write_all(data).await.map_err(|e| e.to_string())?;
    Ok(())
}
