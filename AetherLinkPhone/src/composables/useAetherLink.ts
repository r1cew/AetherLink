import { ref, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";

import {
  scan,
  cancel,
  Format,
  checkPermissions,
  requestPermissions,
} from "@tauri-apps/plugin-barcode-scanner";

// ── Интерфейсы типов ──────────────────────────────────────────────────────────
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

export function useAetherLink() {
  const storePromise = Store.load("settings.json");

  // ── Реактивные переменные (Стейт) ──────────────────────────────────────────────
  const screen = ref<Screen>("servers");
  const servers = ref<Server[]>([]);
  const active = ref<Server | null>(null);
  const profiles = ref<Profile[]>([]);
  const log = ref("");
  const loading = ref(false);
  const isScanning = ref(false);
  const isJustConnected = ref(false);

  const qrText = ref("");
  const phoneName = ref("Мой телефон");
  const pcName = ref("Домашний ПК");
  const shellCmd = ref("");

  async function saveConnectionData() {
    try {
      const store = await storePromise;
      if (active.value) {
        await store.set("activeServer", active.value);
        console.log("✅ Сервер сохранен:", active.value.id);
      }
      if (profiles.value.length > 0) {
        await store.set("profiles", profiles.value);
        console.log("✅ Профили сохранены");
      }
      await store.save();
    } catch (e) {
      console.error("Ошибка сохранения:", e);
    }
  }

  async function loadConnectionData() {
    try {
      const store = await storePromise; // 👈 получаем store из Promise
      const savedServer = await store.get("activeServer");
      const savedProfiles = await store.get("profiles");

      if (savedServer) {
        active.value = savedServer as Server;
        console.log("📦 Загружен сервер:", active.value.name);
      }

      if (savedProfiles) {
        profiles.value = savedProfiles as Profile[];
        console.log("📦 Загружено профилей:", profiles.value.length);
      }

      if (savedServer) {
        isJustConnected.value = true;
        await loadServers();
        if (active.value) {
          await loadProfiles(active.value);
        }
      }
    } catch (e) {
      console.error("Ошибка загрузки:", e);
    }
  }

  loadConnectionData();

  // Вспомогательная функция для логов
  function msg(text: string) {
    log.value = text;
  }

  // ── Логика управления прозрачностью body при сканировании ──────────────────────
  watch(isScanning, (newValue) => {
    if (newValue) {
      document.body.classList.add("scanning-active");
    } else {
      document.body.classList.remove("scanning-active");
    }
  });

  // На случай внезапного демонтирования компонента чистим класс
  onUnmounted(() => {
    document.body.classList.remove("scanning-active");
  });

  // ── Загрузка серверов ─────────────────────────────────────────────────────────
  async function loadServers() {
    try {
      const list = await invoke<Server[]>("get_servers");
      servers.value = list;
    } catch (e) {
      msg(`Ошибка: ${e}`);
    }
  }

  // ── Загрузка профилей автоматизации ───────────────────────────────────────────
  async function loadProfiles(server: Server) {
    try {
      const list = await invoke<Profile[]>("list_profiles", {
        serverId: server.id,
      });
      profiles.value = Array.isArray(list) ? list : [];
    } catch {
      profiles.value = [];
    }
  }

  // ── Действия с серверами ──────────────────────────────────────────────────────
  function selectServer(s: Server) {
    active.value = s;
    log.value = "";
    loadProfiles(s);
    screen.value = "control";
  }

  // ── Авто-подключение после сканирования ───────────────────────────────────────
  async function autoConnect(content: string) {
    loading.value = true;
    msg("Подключение...");
    try {
      const id = await invoke<string>("pair_with_qr", {
        qrJson: content.trim(),
        name: phoneName.value,
        nickname: pcName.value,
      });

      await loadServers();
      const newServer = servers.value.find((s) => s.id === id);

      if (newServer) {
        active.value = newServer;
        log.value = "";
        isJustConnected.value = true;
        await loadProfiles(newServer);
        screen.value = "control";

        await saveConnectionData();
      } else {
        screen.value = "servers";
      }
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      loading.value = false;
    }
  }

  // ── Работа с камерой (QR) ─────────────────────────────────────────────────────
  async function startQrScan() {
    log.value = "";
    try {
      let permission = await checkPermissions();
      if (permission !== "granted") {
        msg("Запрашиваю доступ к камере...");
        permission = await requestPermissions();
      }
      if (permission !== "granted") {
        msg("Доступ к камере отклонен. Включите в настройках телефона.");
        return;
      }

      isScanning.value = true;
      const result = await scan({ formats: [Format.QRCode] });

      isScanning.value = false;

      if (result?.content) {
        await autoConnect(result.content);
      }
    } catch (e) {
      msg(`Ошибка: ${e}`);
    } finally {
      isScanning.value = false;
    }
  }

  async function stopQrScan() {
    try {
      await cancel();
    } catch {}
    isScanning.value = false;
  }

  // ── Ручной паринг через JSON ──────────────────────────────────────────────────
  async function pair() {
    if (!qrText.value.trim()) {
      msg("Сначала отсканируй QR или вставь JSON");
      return;
    }
    loading.value = true;
    try {
      const id = await invoke<string>("pair_with_qr", {
        qrJson: qrText.value.trim(),
        name: phoneName.value,
        nickname: pcName.value,
      });
      msg(`✅ Привязано! ID: ${id.slice(0, 8)}…`);
      qrText.value = "";
      await loadServers();
      screen.value = "servers";
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      loading.value = false;
    }
  }

  // ── Вызовы системных команд (Default Mode) ────────────────────────────────────
  async function safe(command: string, params?: object) {
    if (!active.value) return;
    loading.value = true;
    try {
      const res = await invoke("send_safe", {
        serverId: active.value.id,
        command,
        params: params ?? null,
      });
      msg(`✅ ${command}: ${JSON.stringify(res)}`);
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      loading.value = false;
    }
  }

  // ── Запуск профилей автоматизации ─────────────────────────────────────────────
  async function runProfile(profileId: string, name: string) {
    if (!active.value) return;
    loading.value = true;
    try {
      await invoke("send_run_profile", {
        serverId: active.value.id,
        profileId,
      });
      msg(`✅ Запущено: ${name}`);
    } catch (e) {
      msg(`❌ ${e}`);
    } finally {
      loading.value = false;
    }
  }

  // ── PowerShell терминал ───────────────────────────────────────────────────────
  async function runShell() {
    if (!active.value || !shellCmd.value.trim()) return;
    loading.value = true;
    const currentCmd = shellCmd.value;
    try {
      const res = await invoke<string>("send_shell", {
        serverId: active.value.id,
        cmd: currentCmd,
        shell: "powershell",
      });
      msg(`> ${currentCmd}\n${res}`);
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

  // ── Локальный Beacon поиск ПК ─────────────────────────────────────────────────
  async function discover() {
    if (!active.value) return;
    loading.value = true;
    msg("Ищу ПК в сети...");
    try {
      const ip = await invoke<string>("discover_and_update", {
        serverId: active.value.id,
      });
      msg(`Найден: ${ip}`);
      await loadServers();
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

  return {
    // State
    screen,
    servers,
    active,
    profiles,
    log,
    loading,
    isScanning,
    qrText,
    phoneName,
    pcName,
    shellCmd,
    isJustConnected,

    // Methods
    selectServer,
    startQrScan,
    stopQrScan,
    pair,
    saveConnectionData,
    safe,
    runProfile,
    runShell,
    discover,
    loadServers,
  };
}
