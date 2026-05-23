import { invoke } from "@tauri-apps/api/core";
import type { UseLoggerReturn } from "./useLogger";
import { ref } from "vue";

export function useStartUp(logger: UseLoggerReturn) {
  const startupResult = ref<boolean>(false);

  async function checkStartup() {
    startupResult.value = await invoke("status_startup");
  }

  async function addToStartup() {
    try {
      await invoke("add_to_startup");
      checkStartup();
      await logger.addLog("Автозапуск: включен");
    } catch (e) {
      await logger.addLog(`Произошла ошибка автозапуска: ${e}`);
    }
  }

  async function removeFromStartup() {
    try {
      await invoke("remove_from_startup");
      checkStartup();
      await logger.addLog("Автозапуск: выключен");
    } catch (e) {
      await logger.addLog(`Произошла ошибка автозапуска: ${e}`);
    }
  }

  return {
    addToStartup,
    startupResult,
    removeFromStartup,
    checkStartup,
  };
}

export type UseStartupReturn = ReturnType<typeof useStartUp>;
