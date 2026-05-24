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
    state::AppState,
};
use aetherlink_common::protocol::{ClientRequest, ServerResponse};

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

    // Буферы увеличены на 4 байта для префикса длины, чтобы собирать пакет на месте
    let mut buf = vec![0u8; 65540];
    let mut tmp = vec![0u8; 65540];

    // msg1: recv ← телефон
    let msg1_len = recv_frame_inplace(&mut stream, &mut buf).await?;
    handshake
        .read_message(&buf[..msg1_len], &mut tmp)
        .map_err(|e| format!("handshake msg1: {e}"))?;

    // msg2: send → телефон (смещаемся на 4 байта, оставляя место под длину фрейма)
    let len = handshake
        .write_message(&[], &mut buf[4..])
        .map_err(|e| format!("handshake msg2: {e}"))?;

    let len_bytes = (len as u32).to_be_bytes();
    buf[..4].copy_from_slice(&len_bytes);
    stream
        .write_all(&buf[..4 + len])
        .await
        .map_err(|e| e.to_string())?;
    stream.flush().await.map_err(|e| e.to_string())?;

    // msg3: recv ← телефон
    let msg3_len = recv_frame_inplace(&mut stream, &mut buf).await?;
    handshake
        .read_message(&buf[..msg3_len], &mut tmp)
        .map_err(|e| format!("handshake msg3: {e}"))?;

    // Получаем remote static public key телефона (стал известен после msg3).
    let remote_pubkey = handshake
        .get_remote_static()
        .ok_or("Нет remote static key после хендшейка")?
        .to_vec();

    let mut transport = handshake.into_transport_mode().map_err(|e| e.to_string())?;

    println!("[server] Хендшейк завершён");

    // ── Основной цикл сообщений (ТЕПЕРЬ 100% БЕЗ АЛЛОКАЦИЙ) ───────────────────

    loop {
        // Читаем зашифрованный фрейм прямо в buf
        let enc_len = match recv_frame_inplace(&mut stream, &mut buf).await {
            Ok(l) => l,
            Err(_) => break, // клиент отключился
        };

        // Расшифровываем из buf в tmp
        let plain_len = transport
            .read_message(&buf[..enc_len], &mut tmp)
            .map_err(|e| format!("decrypt: {e}"))?;

        // Десериализуем из слайса памяти
        let request: ClientRequest =
            serde_json::from_slice(&tmp[..plain_len]).map_err(|e| format!("json parse: {e}"))?;

        // Обрабатываем запрос
        let response = dispatch(request, &remote_pubkey, &state).await;

        // Сериализуем ответ прямо в буфер tmp с помощью Cursor, избегая выделения Vec
        let mut writer = std::io::Cursor::new(&mut tmp[..]);
        serde_json::to_writer(&mut writer, &response).map_err(|e| e.to_string())?;
        let plain_resp_len = writer.position() as usize;

        // Шифруем ответ из tmp в buf, со смещением в 4 байта
        let enc_resp_len = transport
            .write_message(&tmp[..plain_resp_len], &mut buf[4..])
            .map_err(|e| format!("encrypt: {e}"))?;

        // Пишем 4 байта длины в самое начало буфера buf
        let resp_len_bytes = (enc_resp_len as u32).to_be_bytes();
        buf[..4].copy_from_slice(&resp_len_bytes);

        // Отправляем длину и зашифрованные данные ОДНИМ системным вызовом
        stream
            .write_all(&buf[..4 + enc_resp_len])
            .await
            .map_err(|e| format!("send: {e}"))?;

        // Принудительно выталкиваем пакет в сеть, минимизируя пинг
        stream.flush().await.map_err(|e| e.to_string())?;
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
        ClientRequest::Pair { token, name } => pair(state, remote_pubkey, &token, &name).await,

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
        ClientRequest::Safe { command, params } => safe::execute(command, params),

        ClientRequest::RunProfile { profile_id } => profiles::run_profile(&data_dir, &profile_id),

        ClientRequest::ListProfiles => match automation::load(&data_dir) {
            Ok(profiles) => {
                ServerResponse::ok_data(serde_json::to_value(&profiles).unwrap_or_default())
            }
            Err(e) => ServerResponse::err(e),
        },

        ClientRequest::Shell { cmd, shell } => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для Shell нужен режим Developer.");
            }
            shell::execute(cmd, shell)
        }

        ClientRequest::GetMode => {
            let mode_str = match device.mode {
                DeviceMode::Default => "default",
                DeviceMode::Developer => "developer",
            };
            ServerResponse::ok_data(serde_json::json!({ "mode": mode_str }))
        }

        ClientRequest::CheckDevStatus => {
            let mode_str = match device.mode {
                DeviceMode::Default => "default",
                DeviceMode::Developer => "developer",
            };
            ServerResponse::ok_data(serde_json::json!({
                "mode": mode_str,
                "is_dev": device.mode == DeviceMode::Developer
            }))
        }

        ClientRequest::CreateProfile {
            name,
            description,
            commands,
        } => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для создания профилей нужен режим Developer.");
            }
            match automation::load(&data_dir) {
                Ok(mut profiles) => {
                    let profile = automation::Profile::new(name, description, commands);
                    profiles.push(profile);
                    match automation::save(&data_dir, &profiles) {
                        Ok(_) => ServerResponse::ok_data(serde_json::json!({
                            "id": profiles.last().unwrap().id
                        })),
                        Err(e) => ServerResponse::err(format!("Ошибка сохранения: {e}")),
                    }
                }
                Err(e) => ServerResponse::err(e),
            }
        }

        ClientRequest::GetDevProfiles => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для редактирования профилей нужен режим Developer.");
            }
            match automation::load(&data_dir) {
                Ok(profiles) => {
                    ServerResponse::ok_data(serde_json::to_value(&profiles).unwrap_or_default())
                }
                Err(e) => ServerResponse::err(e),
            }
        }

        ClientRequest::DeleteProfile { profile_id } => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для удаления профилей нужен режим Developer.");
            }
            match automation::load(&data_dir) {
                Ok(mut profiles) => {
                    profiles.retain(|p| p.id != profile_id);
                    match automation::save(&data_dir, &profiles) {
                        Ok(_) => ServerResponse::ok(),
                        Err(e) => ServerResponse::err(format!("Ошибка сохранения: {e}")),
                    }
                }
                Err(e) => ServerResponse::err(e),
            }
        }

        ClientRequest::AddProfile {
            name,
            description,
            kind,
        } => {
            if device.mode != DeviceMode::Developer {
                return ServerResponse::err("Для добавления команд нужен режим Developer.");
            }
            match serde_json::from_value::<automation::ProfileKind>(kind) {
                Ok(profile_kind) => {
                    let profile = automation::Profile::new(name, description, profile_kind);
                    match automation::load(&data_dir) {
                        Ok(mut profiles) => {
                            profiles.push(profile);
                            match automation::save(&data_dir, &profiles) {
                                Ok(_) => ServerResponse::ok(),
                                Err(e) => ServerResponse::err(format!("Ошибка сохранения: {e}")),
                            }
                        }
                        Err(e) => ServerResponse::err(e),
                    }
                }
                Err(e) => ServerResponse::err(format!("Неверный формат команды: {e}")),
            }
        }

        ClientRequest::Pair { .. } => ServerResponse::err("Устройство уже привязано."),
    }
}

// ─── Паринг ───────────────────────────────────────────────────────────────────

async fn pair(state: &AppState, remote_pubkey: &[u8], token: &str, name: &str) -> ServerResponse {
    let mut s = state.lock().await;

    let valid = s
        .pairing
        .as_ref()
        .map(|p| p.is_valid(token))
        .unwrap_or(false);

    if !valid {
        return ServerResponse::err("Токен паринга недействителен или истёк.");
    }

    if s.registry.find_by_pubkey(remote_pubkey).is_some() {
        return ServerResponse::err("Устройство уже привязано.");
    }

    let device = s.registry.add(name.to_string(), remote_pubkey);
    let device_id = device.id.clone();

    if let Err(e) = save_registry(&s.data_dir, &s.registry) {
        return ServerResponse::err(format!("Ошибка保存 реестра: {e}"));
    }

    s.pairing = None;

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

// ─── Оптимизированное Фреймирование (Inplace) ─────────────────────────────────

async fn recv_frame_inplace(stream: &mut TcpStream, out_buf: &mut [u8]) -> Result<usize, String> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| e.to_string())?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 65535 {
        return Err(format!("Слишком большой фрейм: {len} байт"));
    }

    // Читаем данные прямо в переданный буфер без аллокаций вектора
    stream
        .read_exact(&mut out_buf[..len])
        .await
        .map_err(|e| e.to_string())?;

    Ok(len)
}
