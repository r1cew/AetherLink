import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useAetherLink() {
  // ── состояние ──────────────────────────────────────────────────────────────────
  const qrData = ref("");
  const devices = ref<any[]>([]);
  const profiles = ref<any[]>([]);
  const devMode = ref(false);
  const log = ref<string[]>([]);
  const jsonCheck = ref(false);
  const addPhone = ref(false);

  // ── лог ───────────────────────────────────────────────────────────────────────
  function addLog(msg: string) {
    log.value.unshift(`[${new Date().toLocaleTimeString()}] ${msg}`);
    if (log.value.length > 50) log.value.pop();
  }

  // ── загрузка ──────────────────────────────────────────────────────────────────
  async function loadDevices() {
    try {
      devices.value = await invoke("get_devices");
      addLog(`Устройств: ${devices.value.length}`);
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  async function loadProfiles() {
    try {
      profiles.value = await invoke("get_profiles");
    } catch (e) {
      addLog(`Ошибка профилей: ${e}`);
    }
  }

  // ── паринг ────────────────────────────────────────────────────────────────────
  async function generateQR() {
    try {
      qrData.value = await invoke("generate_pairing_qr");
      addLog("QR сгенерирован — действителен 120 сек");
    } catch (e) {
      addLog(`Ошибка QR: ${e}`);
    }
  }

  // ── устройства ────────────────────────────────────────────────────────────────
  async function setMode(id: string, mode: string) {
    try {
      await invoke("set_device_mode", { deviceId: id, mode });
      addLog(`Режим устройства → ${mode}`);
      loadDevices();
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  async function removeDevice(id: string) {
    try {
      await invoke("remove_device", { deviceId: id });
      addLog("Устройство удалено");
      loadDevices();
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  // ── developer mode ────────────────────────────────────────────────────────────
  async function toggleDevMode() {
    try {
      await invoke("set_developer_mode", { enabled: devMode.value });
      addLog(`Developer Mode: ${devMode.value ? "ВКЛ" : "ВЫКЛ"}`);
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  // ── профили ───────────────────────────────────────────────────────────────────
  const newProfileName = ref("");
  const newProfilePath = ref("");

  async function createTestProfile() {
    if (!newProfileName.value || !newProfilePath.value) return;
    try {
      const id = await invoke("create_profile", {
        name: newProfileName.value,
        description: "Тестовый профиль",
        kind: { type: "run_bat", path: newProfilePath.value },
      });
      addLog(`Профиль создан: ${id}`);
      newProfileName.value = "";
      newProfilePath.value = "";
      loadProfiles();
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  async function deleteProfile(id: string) {
    try {
      await invoke("delete_profile", { profileId: id });
      addLog("Профиль удалён");
      loadProfiles();
    } catch (e) {
      addLog(`Ошибка: ${e}`);
    }
  }

  function showJson() {
    if (qrData.value && qrData.value.trim() !== "") {
      jsonCheck.value = true;
    } else {
      jsonCheck.value = false;
    }
  }

  // ── init ──────────────────────────────────────────────────────────────────────
  onMounted(async () => {
    loadDevices();
    loadProfiles();

    // Слушаем событие паринга с телефона
    await listen("device-paired", (event: any) => {
      addLog(`Телефон привязан: ${event.payload.name}`);
      loadDevices();
    });

    addLog("Сервер запущен на :8080 | Beacon на :9999");
  });

  return {
    // Состояние
    qrData,
    devices,
    profiles,
    devMode,
    addPhone,
    log,
    jsonCheck,
    // Формы
    newProfileName,
    newProfilePath,
    // Методы
    generateQR,
    loadDevices,
    setMode,
    removeDevice,
    toggleDevMode,
    createTestProfile,
    deleteProfile,
    showJson,
  };
}
