import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ref, onMounted, onUnmounted } from "vue";

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
        generateQR();
      }
    }, 1000);
  };

  const stopTimer = () => {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    timerActive.value = false;
  };

  onUnmounted(() => {
    if (timerInterval) {
      clearInterval(timerInterval);
    }
  });

  async function generateQR() {
    try {
      qrData.value = await invoke("generate_pairing_qr");
      addLog("QR сгенерирован — действителен 120 сек");
      timeLeft.value = 120;
      stopTimer();
      startTimer();
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
  const newProfileDescription = ref("");
  const newProfileType = ref("");
  const newProfileArgs = ref<string[]>([]);
  const newProfileScript = ref("");

  async function createProfile() {
    if (!newProfileName.value) return;

    // Сборка объекта "kind" на основании выбранного типа
    let kindPayload: Record<string, any> = {
      type: newProfileType.value,
    };

    if (newProfileType.value === "run_bat") {
      kindPayload.path = newProfilePath.value;
    } else if (newProfileType.value === "run_exe") {
      kindPayload.path = newProfilePath.value;
      kindPayload.args = newProfileArgs.value; // Передаем массив строк
    } else if (newProfileType.value === "power_shell") {
      kindPayload.script = newProfileScript.value; // Только скрипт, без path и args
    }

    try {
      // Отправка структурированных данных в бэкенд Tauri
      const id = await invoke("create_profile", {
        name: newProfileName.value,
        description: newProfileDescription.value,
        kind: kindPayload,
      });

      addLog(`Профиль создан: ${id}`);

      // Очистка состояния стейта в composable
      newProfileName.value = "";
      newProfilePath.value = "";
      newProfileArgs.value = [];
      newProfileScript.value = "";

      loadProfiles();
    } catch (e) {
      addLog(`Ошибка: ${e}`);
      throw e; // Пробрасываем ошибку для обработки в handleCreateTask
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
    timeLeft,
    formatTime,
    // Формы
    newProfileName,
    newProfilePath,
    newProfileDescription,
    newProfileType,
    newProfileArgs,
    newProfileScript,
    // Методы
    generateQR,
    addLog,
    loadDevices,
    setMode,
    removeDevice,
    toggleDevMode,
    createProfile,
    deleteProfile,
    showJson,
  };
}
