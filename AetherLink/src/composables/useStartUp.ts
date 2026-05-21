import { invoke } from "@tauri-apps/api/core";
import type { UseLoggerReturn } from "./useLogger";

export function useStartUp(logger: UseLoggerReturn) {
  async function addToStartup() {
    try {
      await invoke("add_to_startup");
      await logger.addLog("Автозапуск приложения включён");
    } catch (e) {
      await logger.addLog(`Произошла ошибка автозапуска: ${e}`);
    }
  }

  async function removeFromStartup() {
    try {
      await invoke("remove_from_startup");
      await logger.addLog("Автозапуск приложения выключён");
    } catch (e) {
      await logger.addLog(`Произошла ошибка автозапуска: ${e}`);
    }
  }

  return {
    addToStartup,
    removeFromStartup,
  };
}

export type UseStartupReturn = ReturnType<typeof useStartUp>;
