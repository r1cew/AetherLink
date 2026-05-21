import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useLogger } from "./useLogger";
import { useDeviceManager } from "./useDeviceManager";
import { useProfileManager } from "./useProfileManager";
import { useQRPairing } from "./useQRPairing";

let initialized = false;

export function useAetherLink() {
  // Вспомогательные состояния
  const addPhone = ref(false);
  const jsonCheck = ref(false);
  const devMode = ref(false);

  // Инициализация модулей
  const logger = useLogger();
  const deviceManager = useDeviceManager(logger);
  const profileManager = useProfileManager(logger);
  const qrPairing = useQRPairing(logger);

  // Обертка для toggleDevMode с сохранением состояния
  async function toggleDevMode() {
    await deviceManager.toggleDevMode(devMode.value);
  }

  function showJson() {
    if (qrPairing.qrData.value && qrPairing.qrData.value.trim() !== "") {
      jsonCheck.value = true;
    } else {
      jsonCheck.value = false;
    }
  }

  onMounted(async () => {
    if (initialized) return;
    initialized = true;

    await deviceManager.loadDevices();
    await profileManager.loadProfiles();
    await logger.getLogs();

    const unlisten = await listen("device-paired", async () => {
      await deviceManager.loadDevices();
    });

    onUnmounted(() => {
      unlisten();
    });
  });

  return {
    // Состояния
    addPhone,
    jsonCheck,
    devMode,

    // Логи
    logData: logger.logData,
    get_logs: logger.getLogs,
    add_log: logger.addLog,

    // Устройства
    devices: deviceManager.devices,
    loadDevices: deviceManager.loadDevices,
    setMode: deviceManager.setMode,
    removeDevice: deviceManager.removeDevice,

    // Профили
    profiles: profileManager.profiles,
    newProfileName: profileManager.newProfileName,
    newProfilePath: profileManager.newProfilePath,
    newProfileDescription: profileManager.newProfileDescription,
    newProfileType: profileManager.newProfileType,
    newProfileArgs: profileManager.newProfileArgs,
    newProfileScript: profileManager.newProfileScript,
    loadProfiles: profileManager.loadProfiles,
    runProfile: profileManager.runProfile,
    createProfile: profileManager.createProfile,
    deleteProfile: profileManager.deleteProfile,

    // QR
    qrData: qrPairing.qrData,
    timeLeft: qrPairing.timeLeft,
    formatTime: qrPairing.formatTime,
    generateQR: qrPairing.generateQR,

    // Утилиты
    toggleDevMode,
    showJson,
  };
}
