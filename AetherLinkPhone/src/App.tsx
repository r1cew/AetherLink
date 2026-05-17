import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
// Импортируем сканер из плагина Tauri
import {
  scan,
  cancel,
  Format,
  checkPermissions,
  requestPermissions,
} from "@tauri-apps/plugin-barcode-scanner";
import "./App.css";

// ── типы ───────────────────────────────────────────────────────────────────────
interface Server {
  id: string;
  name: string;
  ip: string;
  port: number;
}
interface Profile {
  id: string;
  name: string;
  description?: string;
}

type Screen = "servers" | "pair" | "control";

export default function App() {
  const [screen, setScreen] = useState<Screen>("servers");
  const [servers, setServers] = useState<Server[]>([]);
  const [active, setActive] = useState<Server | null>(null);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [log, setLog] = useState("");
  const [loading, setLoading] = useState(false);
  const [isScanning, setIsScanning] = useState(false); // Стейт для режима камеры

  // ── паринг форма ─────────────────────────────────────────────────────────────
  const [qrText, setQrText] = useState("");
  const [phoneName, setPhoneName] = useState("Мой телефон");
  const [pcName, setPcName] = useState("Домашний ПК");

  // ── shell ────────────────────────────────────────────────────────────────────
  const [shellCmd, setShellCmd] = useState("");

  function msg(text: string) {
    setLog(text);
  }

  // ── загрузить серверы ─────────────────────────────────────────────────────────
  async function loadServers() {
    try {
      const list = await invoke<Server[]>("get_servers");
      setServers(list);
    } catch (e) {
      msg(`Ошибка: ${e}`);
    }
  }

  useEffect(() => {
    loadServers();
  }, []);

  // Управляем прозрачностью body в зависимости от стейта сканирования
  useEffect(() => {
    if (isScanning) {
      document.body.classList.add("scanning-active");
    } else {
      document.body.classList.remove("scanning-active");
    }
    return () => document.body.classList.remove("scanning-active");
  }, [isScanning]);

  // ── загрузить профили ─────────────────────────────────────────────────────────
  async function loadProfiles(server: Server) {
    try {
      const list = await invoke<Profile[]>("list_profiles", {
        serverId: server.id,
      });
      setProfiles(Array.isArray(list) ? list : []);
    } catch {
      setProfiles([]);
    }
  }

  // ── выбрать сервер ────────────────────────────────────────────────────────────
  function selectServer(s: Server) {
    setActive(s);
    setLog("");
    loadProfiles(s);
    setScreen("control");
  }

  // ── функция запуска камеры ─────────────────────────────────────────────────────
  async function startQrScan() {
    setLog("");
    try {
      // 1. Проверяем текущий статус разрешения в Android
      let permission = await checkPermissions();

      // 2. Если разрешение еще не выдано (статус 'prompt' или 'prompt-with-rationale')
      if (permission !== "granted") {
        msg("Запрашиваю доступ к камере...");
        permission = await requestPermissions(); // Вот тут Android покажет системное окно
      }

      // 3. Если пользователь всё-таки отказал в доступе
      if (permission !== "granted") {
        msg(
          "❌ Доступ к камере отклонен. Вы можете включить его вручную в настройках телефона.",
        );
        return;
      }

      // 4. Если доступ успешно получен (или уже был) — включаем камеру
      setIsScanning(true);
      const result = await scan({
        formats: [Format.QRCode],
      });

      if (result && result.content) {
        setQrText(result.content);
        msg("QR-код успешно считан!");
      }
    } catch (e) {
      msg(`Ошибка камеры: ${e}`);
    } finally {
      setIsScanning(false);
    }
  }

  // ── принудительная отмена камеры ────────────────────────────────────────────────
  async function stopQrScan() {
    try {
      await cancel();
    } catch {}
    setIsScanning(false);
  }

  // ── паринг ───────────────────────────────────────────────────────────────────
  async function pair() {
    if (!qrText.trim()) {
      msg("Сначала отсканируй QR или вставь JSON");
      return;
    }
    setLoading(true);
    try {
      const id = await invoke<string>("pair_with_qr", {
        qrJson: qrText.trim(),
        name: phoneName,
        nickname: pcName,
      });
      msg(`✅ Привязано! ID: ${id.slice(0, 8)}…`);
      setQrText("");
      await loadServers();
      setScreen("servers");
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // ── safe команды ──────────────────────────────────────────────────────────────
  async function safe(command: string, params?: object) {
    if (!active) return;
    setLoading(true);
    try {
      const res = await invoke("send_safe", {
        serverId: active.id,
        command,
        params: params ?? null,
      });
      msg(`✅ ${command}: ${JSON.stringify(res)}`);
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // ── запуск профиля ────────────────────────────────────────────────────────────
  async function runProfile(profileId: string, name: string) {
    if (!active) return;
    setLoading(true);
    try {
      await invoke("send_run_profile", { serverId: active.id, profileId });
      msg(`✅ Запущено: ${name}`);
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // ── shell ─────────────────────────────────────────────────────────────────────
  async function runShell() {
    if (!active || !shellCmd.trim()) return;
    setLoading(true);
    try {
      const res = await invoke<string>("send_shell", {
        serverId: active.id,
        cmd: shellCmd,
        shell: "powershell",
      });
      msg(`> ${shellCmd}\n${res}`);
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // ── beacon fallback ────────────────────────────────────────────────────────────
  async function discover() {
    if (!active) return;
    setLoading(true);
    msg("Ищу ПК в сети...");
    try {
      const ip = await invoke<string>("discover_and_update", {
        serverId: active.id,
      });
      msg(`✅ Найден: ${ip}`);
      loadServers();
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      setLoading(false);
    }
  }

  // ─────────────────────────────────────────────────────────────────────────────
  // RENDER
  // ─────────────────────────────────────────────────────────────────────────────

  // ── экран: список серверов ────────────────────────────────────────────────────
  if (screen === "servers")
    return (
      <main className="shell">
        <div className="topbar">
          <span className="app-title">⚡ AetherLink</span>
          <button className="icon-btn" onClick={() => setScreen("pair")}>
            ＋
          </button>
        </div>

        {servers.length === 0 ? (
          <div className="empty-state">
            <p>Нет привязанных ПК</p>
            <button className="btn-primary" onClick={() => setScreen("pair")}>
              Привязать ПК
            </button>
          </div>
        ) : (
          <div className="server-list">
            {servers.map((s) => (
              <div
                key={s.id}
                className="server-card"
                onClick={() => selectServer(s)}
              >
                <div className="server-icon">🖥</div>
                <div className="server-info">
                  <div className="server-name">{s.name}</div>
                  <div className="server-addr">
                    {s.ip}:{s.port}
                  </div>
                </div>
                <div className="chevron">›</div>
              </div>
            ))}
          </div>
        )}
      </main>
    );

  // ── экран: паринг ─────────────────────────────────────────────────────────────
  if (screen === "pair")
    return (
      <main className="shell">
        {/* Оверлей сканера камеры, который рендерится поверх пустогоWebView */}
        {isScanning && (
          <div className="camera-overlay">
            <div className="scanner-hint">Наведите камеру на QR-код на ПК</div>
            <div className="scanner-target"></div>
            <button
              className="btn-primary"
              style={{ background: "var(--red)" }}
              onClick={stopQrScan}
            >
              Отмена
            </button>
          </div>
        )}

        <div className="topbar">
          <button className="icon-btn" onClick={() => setScreen("servers")}>
            ‹
          </button>
          <span className="app-title">Привязать ПК</span>
          <span />
        </div>

        <div className="pair-screen">
          <div
            className="pair-icon"
            style={{ cursor: "pointer" }}
            onClick={startQrScan}
          >
            📷
          </div>
          <p className="pair-hint">
            Нажмите на иконку камеры, чтобы отсканировать QR на ПК.
          </p>

          <button
            className="btn-primary"
            onClick={startQrScan}
            style={{ marginBottom: "10px" }}
          >
            [📷] Начать сканирование
          </button>

          <hr style={{ borderColor: "var(--border)", margin: "8px 0" }} />

          <label className="field-label">Или вставьте JSON вручную:</label>
          <textarea
            className="qr-input"
            placeholder='{"ip":"192.168.1.5","port":8080,...}'
            value={qrText}
            onChange={(e) => setQrText(e.target.value)}
            rows={3}
          />

          <label className="field-label">Имя телефона (видно на ПК)</label>
          <input
            className="text-input"
            value={phoneName}
            onChange={(e) => setPhoneName(e.target.value)}
          />

          <label className="field-label">Название ПК (видно тебе)</label>
          <input
            className="text-input"
            value={pcName}
            onChange={(e) => setPcName(e.target.value)}
          />

          <button
            className="btn-primary"
            onClick={pair}
            disabled={loading || !qrText.trim()}
          >
            {loading ? "Подключение…" : "Привязать"}
          </button>

          {log && <div className="log-box">{log}</div>}
        </div>
      </main>
    );

  // ── экран: управление ─────────────────────────────────────────────────────────
  return (
    <main className="shell">
      <div className="topbar">
        <button className="icon-btn" onClick={() => setScreen("servers")}>
          ‹
        </button>
        <span className="app-title">{active?.name}</span>
        <button className="icon-btn" onClick={discover} title="Найти по beacon">
          📡
        </button>
      </div>

      {/* Safe Mode */}
      <section className="section">
        <div className="section-title">⚡ Safe Mode</div>

        <div className="grid-2">
          <button className="ctrl-btn danger" onClick={() => safe("shutdown")}>
            <span>⏻</span>Выключить
          </button>
          <button className="ctrl-btn" onClick={() => safe("sleep")}>
            <span>🌙</span>Сон
          </button>
          <button className="ctrl-btn" onClick={() => safe("lock")}>
            <span>🔒</span>Блокировка
          </button>
        </div>

        <div className="section-subtitle">Звук</div>
        <div className="grid-3">
          <button className="ctrl-btn sm" onClick={() => safe("volume_down")}>
            🔉
          </button>
          <button
            className="ctrl-btn sm"
            onClick={() => safe("volume_set", { level: 50 })}
          >
            50%
          </button>
          <button className="ctrl-btn sm" onClick={() => safe("volume_up")}>
            🔊
          </button>
        </div>

        <div className="section-subtitle">Медиа</div>
        <div className="grid-4">
          <button className="ctrl-btn sm" onClick={() => safe("media_prev")}>
            ⏮
          </button>
          <button className="ctrl-btn sm" onClick={() => safe("media_play")}>
            ⏯
          </button>
          <button className="ctrl-btn sm" onClick={() => safe("media_pause")}>
            ⏸
          </button>
          <button className="ctrl-btn sm" onClick={() => safe("media_next")}>
            ⏭
          </button>
        </div>
      </section>

      {/* Automation Mode */}
      {profiles.length > 0 && (
        <section className="section">
          <div className="section-title">🤖 Automation</div>
          {profiles.map((p) => (
            <button
              key={p.id}
              className="profile-btn"
              onClick={() => runProfile(p.id, p.name)}
            >
              <span className="profile-name">{p.name}</span>
              {p.description && (
                <span className="profile-desc">{p.description}</span>
              )}
            </button>
          ))}
        </section>
      )}

      {/* Developer Mode */}
      <section className="section">
        <div className="section-title">🛠 Developer Shell</div>
        <div className="shell-row">
          <input
            className="text-input shell-input"
            placeholder="powershell команда..."
            value={shellCmd}
            onChange={(e) => setShellCmd(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && runShell()}
          />
          <button className="btn-send" onClick={runShell} disabled={loading}>
            ▶
          </button>
        </div>
      </section>

      {/* Лог */}
      {log && (
        <div className="log-box" onClick={() => setLog("")}>
          <pre>{log}</pre>
          <small>тап чтобы закрыть</small>
        </div>
      )}

      {loading && <div className="loading-bar" />}
    </main>
  );
}
