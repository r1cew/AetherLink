// composables/useLogger.ts
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const logData = ref<string[]>([]);

export function useLogger() {
  async function getLogs() {
    try {
      logData.value = await invoke("get_logs");
    } catch (e) {
      console.error("Failed to get logs:", e);
    }
  }

  async function addLog(message: string) {
    try {
      await invoke("add_log", { message });
      await getLogs();
    } catch (e) {
      console.error("Failed to add log:", e);
    }
  }

  return {
    logData,
    getLogs,
    addLog,
  };
}

export type UseLoggerReturn = ReturnType<typeof useLogger>;
