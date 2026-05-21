// composables/useDeviceManager.ts
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UseLoggerReturn } from "./useLogger";

export interface Device {
  id: string;
  name: string;
  mode: string;
}

export function useDeviceManager(logger: UseLoggerReturn) {
  const devices = ref<Device[]>([]);

  async function loadDevices() {
    try {
      devices.value = await invoke("get_devices");
      await logger.addLog(`Подключенных устройств: ${devices.value.length}`);
    } catch (e) {
      await logger.addLog(`Ошибка загрузки устройств: ${e}`);
      console.error("Failed to load devices:", e);
      devices.value = [];
    }
  }

  async function setMode(id: string, mode: string, name: string) {
    try {
      await invoke("set_device_mode", { deviceId: id, mode });
      await logger.addLog(`Устройство "${name}": режим изменён на ${mode}`);
      await loadDevices();
    } catch (e) {
      await logger.addLog(`Ошибка смены режима для ${id}: ${e}`);
      console.error("Failed to set mode:", e);
    }
  }

  async function removeDevice(id: string) {
    try {
      await invoke("remove_device", { deviceId: id });
      await logger.addLog(`Устройство ${id} удалено`);
      await loadDevices();
    } catch (e) {
      await logger.addLog(`Ошибка удаления устройства ${id}: ${e}`);
      console.error("Failed to remove device:", e);
    }
  }

  async function toggleDevMode(devMode: boolean) {
    try {
      await invoke("set_developer_mode", { enabled: devMode });
      await logger.addLog(
        `Developer mode: ${devMode ? "включён" : "выключен"}`,
      );
    } catch (e) {
      await logger.addLog(`Ошибка переключения dev mode: ${e}`);
      console.error("Failed to toggle dev mode:", e);
    }
  }

  return {
    devices,
    loadDevices,
    setMode,
    removeDevice,
    toggleDevMode,
  };
}

export type UseDeviceManagerReturn = ReturnType<typeof useDeviceManager>;
