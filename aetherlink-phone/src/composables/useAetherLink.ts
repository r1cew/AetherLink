import { ref } from "vue";
import { useRouter } from "vue-router";
import { useNotification } from "./useNotification";
import type { Server, Profile, DevStatus, NewTask } from "./types";
import {
  saveConnectionData,
  loadConnectionData,
  resetConnectionData,
} from "./store";
import * as api from "./api";

const screen = ref<string>("servers");
const servers = ref<Server[]>([]);
const active = ref<Server | null>(null);
const profiles = ref<Profile[]>([]);
const log = ref("");
const loading = ref(false);
const isScanning = ref(false);
const isJustConnected = ref(false);
const devStatus = ref<DevStatus | null>(null);
const jsonAuth = ref<string>("");
const qrText = ref("");
const phoneName = ref("");
const pcName = ref("");
const shellCmd = ref("");
const newTask = ref<NewTask>({
  name: "",
  description: "",
  path: "",
  type: "run_bat",
});

export function useAetherLink() {
  const { success, error, warning, info } = useNotification();
  const router = useRouter();

  async function initialize() {
    const { activeServer, profiles: savedProfiles } =
      await loadConnectionData();
    if (activeServer) active.value = activeServer;
    if (savedProfiles.length) profiles.value = savedProfiles;

    if (activeServer) {
      isJustConnected.value = true;
      await loadServers();
      if (active.value) await loadProfiles(active.value);
    }
  }

  function msg(text: string) {
    log.value = text;
  }

  async function loadServers() {
    try {
      servers.value = await api.getServers();
    } catch (e) {
      error(`${e}`);
    }
  }

  async function loadProfiles(server: Server) {
    try {
      const list = await api.listProfiles(server.id);
      profiles.value = Array.isArray(list) ? list : [];
    } catch {
      profiles.value = [];
    }
  }

  async function selectServer(s: Server) {
    active.value = s;
    log.value = "";
    screen.value = "control";
    isJustConnected.value = false;

    info(`Подключение к ${s.name || s.id.slice(0, 8)}...`, 2000);

    try {
      await loadProfiles(s);
      await saveConnectionData(active.value, profiles.value);

      success(`Подключен к серверу: ${s.name || s.id.slice(0, 8)}`, 3000);
      isJustConnected.value = true;
      router.push("/main");
    } catch (err) {
      error(`Ошибка подключения: ${err}`, 4000);
    }
  }

  async function JsonLogin() {
    if (!jsonAuth.value.trim()) return;
    try {
      const jsonData = JSON.parse(jsonAuth.value.trim());
      await autoConnect(JSON.stringify(jsonData));
      router.push("/main");
      jsonAuth.value = "";
    } catch (error) {
      console.error("Ошибка парсинга JSON:", error);
    }
  }

  async function autoConnect(content: string) {
    loading.value = true;
    msg("Подключение к ПК...");
    try {
      const id = await api.pairWithQR(
        content.trim(),
        phoneName.value || "Мой телефон",
        pcName.value || "Домашний ПК",
      );
      await stopQrScan();
      await loadServers();
      const newServer = servers.value.find((s) => s.id === id);

      if (newServer) {
        active.value = newServer;
        log.value = "";
        isJustConnected.value = true;
        success("Успешное подключение");
        await loadProfiles(newServer);
        screen.value = "control";
        await saveConnectionData(active.value, profiles.value);
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
    success("Отсканируйте QR c ПК");
    isScanning.value = true;
  }

  function stopQrScan() {
    isScanning.value = false;
  }

  async function pair() {
    if (!qrText.value.trim()) return msg("Вставь JSON или отсканируй QR");
    loading.value = true;
    try {
      const id = await api.pairWithQR(
        qrText.value.trim(),
        phoneName.value,
        pcName.value,
      );
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

  async function checkDev() {
    if (!active.value) return;
    loading.value = true;
    try {
      const dev = await api.checkDevStatus(active.value.id);
      devStatus.value = dev;
      console.log(dev);
    } catch (e) {
      console.log(e);
      devStatus.value = null;
    } finally {
      loading.value = false;
    }
  }

  async function safe(command: string, params?: object) {
    if (!active.value) {
      warning("Нет активного сервера", 2000);
      return;
    }

    loading.value = true;

    try {
      const res = await api.sendSafe(active.value.id, command, params);
      const resultMsg = `${command}: выполнено успешно`;
      msg(resultMsg);
      success(resultMsg, 2000);

      if (params) {
        info(`Параметры: ${JSON.stringify(params)}`, 1500);
      }
    } catch (err) {
      const errorMsg = `Ошибка выполнения ${command}: ${err}`;
      msg(errorMsg);
      error(errorMsg, 4000);
    } finally {
      setTimeout(() => {
        loading.value = false;
      }, 2000);
    }
  }

  async function runProfile(profileId: string, profileName: string) {
    if (!active.value) return;
    loading.value = true;
    info("Запуск профиля...", 2000);
    try {
      await api.runProfile(active.value.id, profileId);
      success(`Профиль ${profileName} успешно запущен`, 3000);
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      setTimeout(() => {
        loading.value = false;
      }, 2000);
    }
  }

  async function createProfile() {
    if (!active.value) return;
    try {
      await api.createProfile(
        active.value.id,
        newTask.value.name || "Название не указано",
        newTask.value.description || undefined,
        { type: newTask.value.type, path: newTask.value.path },
      );
    } catch (e) {
      msg(`Ошибка: ${e}`);
    }
  }

  async function discover() {
    if (!active.value) return;
    loading.value = true;
    msg("Ищу ПК в сети...");
    try {
      await api.discoverAndUpdate(active.value.id);
      await loadServers();
    } catch (e) {
      msg(`Ошибка ${e}`);
    } finally {
      loading.value = false;
    }
  }

  async function removeServer(serverId: string) {
    try {
      await api.removeServer(serverId);
      loadServers();
    } catch (e) {
      console.log(e);
    }
  }

  async function resetConnection() {
    await resetConnectionData();
    active.value = null;
    profiles.value = [];
    screen.value = "servers";
    isJustConnected.value = false;
    await loadServers();
  }

  // Инициализация
  initialize();

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
    devStatus,
    jsonAuth,
    isJustConnected,
    JsonLogin,
    selectServer,
    startQrScan,
    stopQrScan,
    createProfile,
    checkDev,
    pair,
    saveConnectionData: () => saveConnectionData(active.value, profiles.value),
    safe,
    runProfile,
    discover,
    loadServers,
    loadProfiles,
    removeServer,
    autoConnect,
    resetConnection,
  };
}
