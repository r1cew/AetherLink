import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ── состояние ──────────────────────────────────────────────────────────────────
const qrData = ref("");
const devices = ref<any[]>([]);
const profiles = ref<any[]>([]);
const devMode = ref(false);
const jsonCheck = ref(false);
const addPhone = ref(false);
const logData = ref<string[]>([]);
let initialized = false;

export function useAetherLink() {
  // Логи
  async function get_logs() {
    try {
      logData.value = await invoke("get_logs");
    } catch (e) {
      console.error("Failed to get logs:", e);
    }
  }

  async function add_log(message: string) {
    try {
      await invoke("add_log", { message });
      await get_logs();
    } catch (e) {
      console.error("Failed to add log:", e);
    }
  }

  // ── загрузка и запуск профиля ──────────────────────────────────────────────────────────────────
  async function loadDevices() {
    try {
      devices.value = await invoke("get_devices");
      await add_log(`Подключенных устройств: ${devices.value.length}`);
    } catch (e) {
      await add_log(`Ошибка загрузки устройств: ${e}`);
      console.error("Failed to load devices:", e);
      devices.value = [];
    }
  }

  async function loadProfiles() {
    try {
      profiles.value = await invoke("get_profiles");
      await add_log(`Профилей: ${profiles.value.length}`);
    } catch (e) {
      await add_log(`Ошибка загрузки профилей: ${e}`);
      console.error("Failed to load profiles:", e);
    }
  }

  async function runProfile(id: string, name: string) {
    try {
      await invoke("run_profile_local", { profileId: id });
      add_log(`Профиль "${name}" запущен`);
    } catch (e) {
      add_log(`Ошибка: ${e}`);
    }
  }

  // ── паринг ────────────────────────────────────────────────────────────────────
  const timeLeft = ref(120);
  const timerActive = ref(false);
  let timerInterval: number | null = null;

  const formatTime = (seconds: number): string => {
    return seconds.toString();
  };

  const startTimer = () => {
    if (timerActive.value) return;
    if (timeLeft.value <= 0) {
      return;
    }

    timerActive.value = true;
    timerInterval = window.setInterval(() => {
      if (timeLeft.value > 0) {
        timeLeft.value--;
      } else {
        stopTimer();
      }
    }, 1000);
  };

  const stopTimer = () => {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
      qrData.value = "";
    }
    timerActive.value = false;
  };

  async function generateQR() {
    try {
      qrData.value = await invoke("generate_pairing_qr");
      await add_log(`QR код сгенерирован`);
      timeLeft.value = 120;
      stopTimer();
      startTimer();
    } catch (e) {
      await add_log(`Ошибка генерации QR: ${e}`);
      console.error("QR generation error:", e);
    }
  }

  // ── устройства ────────────────────────────────────────────────────────────────
  async function setMode(id: string, mode: string, name: string) {
    try {
      await invoke("set_device_mode", { deviceId: id, mode });
      await add_log(`Устройство "${name}": режим изменён на ${mode}`);
      await loadDevices();
    } catch (e) {
      await add_log(`Ошибка смены режима для ${id}: ${e}`);
      console.error("Failed to set mode:", e);
    }
  }

  async function removeDevice(id: string) {
    try {
      await invoke("remove_device", { deviceId: id });
      await add_log(`Устройство ${id} удалено`);
      await loadDevices();
    } catch (e) {
      await add_log(`Ошибка удаления устройства ${id}: ${e}`);
      console.error("Failed to remove device:", e);
    }
  }

  // ── developer mode ────────────────────────────────────────────────────────────
  async function toggleDevMode() {
    try {
      await invoke("set_developer_mode", { enabled: devMode.value });
      await add_log(
        `Developer mode: ${devMode.value ? "включён" : "выключен"}`,
      );
    } catch (e) {
      await add_log(`Ошибка переключения dev mode: ${e}`);
      console.error("Failed to toggle dev mode:", e);
    }
  }

  // ── профили ───────────────────────────────────────────────────────────────────
  const newProfileName = ref("");
  const newProfilePath = ref("");
  const newProfileDescription = ref("");
  const newProfileType = ref("");
  const newProfileArgs = ref<string[]>([]);
  const newProfileScript = ref("");

  async function createProfile() {
    if (!newProfileName.value) return;

    let kindPayload: Record<string, any> = {
      type: newProfileType.value,
    };

    if (newProfileType.value === "run_bat") {
      kindPayload.path = newProfilePath.value;
    } else if (newProfileType.value === "run_exe") {
      kindPayload.path = newProfilePath.value;
      kindPayload.args = newProfileArgs.value;
    } else if (newProfileType.value === "power_shell") {
      kindPayload.script = newProfileScript.value;
    }

    try {
      await invoke("create_profile", {
        name: newProfileName.value,
        description: newProfileDescription.value,
        kind: kindPayload,
      });

      await add_log(
        `Профиль создан: ${newProfileName.value} (${newProfileType.value})`,
      );

      newProfileName.value = "";
      newProfilePath.value = "";
      newProfileArgs.value = [];
      newProfileScript.value = "";

      await loadProfiles();
    } catch (e) {
      await add_log(`Ошибка создания профиля: ${e}`);
      console.error("Failed to create profile:", e);
      throw e;
    }
  }

  async function deleteProfile(id: string, name: string) {
    try {
      await invoke("delete_profile", { profileId: id });
      await add_log(`Профиль "${name}" удалён`);
      await loadProfiles();
    } catch (e) {
      await add_log(`Ошибка удаления профиля ${id}: ${e}`);
      console.error("Failed to delete profile:", e);
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
  let unlisten: (() => void) | null = null;

  onMounted(async () => {
    if (initialized) return;

    initialized = true;

    await loadDevices();
    await loadProfiles();
    await get_logs();

    unlisten = await listen("device-paired", async () => {
      await loadDevices();
    });
  });

  onUnmounted(() => {
    stopTimer();
  });

  return {
    // Состояние
    qrData,
    devices,
    profiles,
    devMode,
    addPhone,
    jsonCheck,
    timeLeft,
    formatTime,
    // Формы
    newProfileName,
    newProfilePath,
    newProfileDescription,
    newProfileType,
    newProfileArgs,
    newProfileScript,
    logData,
    // Методы
    generateQR,
    loadProfiles,
    loadDevices,
    setMode,
    removeDevice,
    toggleDevMode,
    createProfile,
    get_logs,
    runProfile,
    add_log,
    deleteProfile,
    showJson,
  };
}
