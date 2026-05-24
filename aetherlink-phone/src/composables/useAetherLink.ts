import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";

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

// type Screen = "servers" | "pair" | "control";

// ── Глобальный стейт приложения (вынесен за пределы функции для совместного использования) ──────────────────
const screen = ref<string>("servers");
const servers = ref<Server[]>([]);
const active = ref<Server | null>(null);
const profiles = ref<Profile[]>([]);
const log = ref("");
const loading = ref(false);
const isScanning = ref(false);
const isJustConnected = ref(false);
const error = ref<string>("");

const qrText = ref("");
const phoneName = ref("Мой телефон");
const pcName = ref("Домашний ПК");
const shellCmd = ref("");

const newTask = ref({
  name: "",
  description: "",
  type: "run_bat",
  path: "",
  args: "",
  script: "", // Для PowerShell
});

export function useAetherLink() {
  const storePromise = Store.load("settings.json");

  // ── Логика работы с памятью ──────────────────────────────────────────────────
  async function saveConnectionData() {
    try {
      const store = await storePromise;
      if (active.value) await store.set("activeServer", active.value);
      if (profiles.value.length > 0)
        await store.set("profiles", profiles.value);
      await store.save();
    } catch (e) {
      console.error("Ошибка сохранения:", e);
    }
  }

  async function loadConnectionData() {
    try {
      const store = await storePromise;
      const savedServer = await store.get("activeServer");
      const savedProfiles = await store.get("profiles");

      if (savedServer) active.value = savedServer as Server;
      if (savedProfiles) profiles.value = savedProfiles as Profile[];

      if (savedServer) {
        isJustConnected.value = true;
        await loadServers();
        if (active.value) await loadProfiles(active.value);
      }
    } catch (e) {
      console.error("Ошибка загрузки:", e);
    }
  }

  // Очистка данных (разрыв связки с ПК)
  async function resetConnection() {
    try {
      const store = await storePromise;
      await store.delete("activeServer");
      await store.delete("profiles");
      await store.save();
      active.value = null;
      profiles.value = [];
      screen.value = "servers";
      isJustConnected.value = false;
      await loadServers();
    } catch (e) {
      console.error("Ошибка сброса:", e);
    }
  }

  loadConnectionData();

  function msg(text: string) {
    log.value = text;
  }

  async function loadServers() {
    try {
      servers.value = await invoke<Server[]>("get_servers");
    } catch (e) {
      msg(`Ошибка: ${e}`);
    }
  }

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

  function selectServer(s: Server) {
    active.value = s;
    log.value = "";
    loadProfiles(s);
    screen.value = "control";
  }

  async function autoConnect(content: string) {
    loading.value = true;
    msg("Подключение к ПК...");
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
      msg(`Ошибка сопряжения: ${e}`);
      screen.value = "servers";
    } finally {
      loading.value = false;
    }
  }

  function startQrScan() {
    log.value = "";
    isScanning.value = true;
  }

  function stopQrScan() {
    isScanning.value = false;
  }

  async function pair() {
    if (!qrText.value.trim()) return msg("Вставь JSON или отсканируй QR");
    loading.value = true;
    try {
      const id = await invoke<string>("pair_with_qr", {
        qrJson: qrText.value.trim(),
        name: phoneName.value,
        nickname: pcName.value,
      });
      msg(`Привязано! ID: ${id.slice(0, 8)}…`);
      qrText.value = "";
      await loadServers();
      screen.value = "servers";
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

  async function safe(command: string, params?: object) {
    if (!active.value) return;
    loading.value = true;
    try {
      const res = await invoke("send_safe", {
        serverId: active.value.id,
        command,
        params: params ?? null,
      });
      msg(`${command}: ${JSON.stringify(res)}`);
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

  async function runProfile(profileId: string) {
    if (!active.value) return;
    loading.value = true;
    try {
      await invoke("send_run_profile", {
        serverId: active.value.id,
        profileId,
      });
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

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
    newTask,
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
    selectServer,
    startQrScan,
    stopQrScan,
    pair,
    saveConnectionData,
    safe,
    runProfile,
    discover,
    loadServers,
    loadProfiles,
    autoConnect,
    resetConnection,
  };
}
