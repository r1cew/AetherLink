// composables/useProfileManager.ts
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UseLoggerReturn } from "./useLogger";

export interface Profile {
  id: string;
  name: string;
  description: string;
  type: string;
}

export function useProfileManager(logger: UseLoggerReturn) {
  const profiles = ref<Profile[]>([]);

  const newProfileName = ref("");
  const newProfilePath = ref("");
  const newProfileDescription = ref("");
  const newProfileType = ref("");
  const newProfileArgs = ref<string[]>([]);
  const newProfileScript = ref("");

  async function loadProfiles() {
    try {
      profiles.value = await invoke("get_profiles");
      await logger.addLog(`Профилей: ${profiles.value.length}`);
    } catch (e) {
      await logger.addLog(`Ошибка загрузки профилей: ${e}`);
      console.error("Failed to load profiles:", e);
    }
  }

  async function runProfile(id: string, name: string) {
    try {
      await invoke("run_profile_local", { profileId: id });
      logger.addLog(`Профиль "${name}" запущен`);
    } catch (e) {
      logger.addLog(`Ошибка: ${e}`);
    }
  }

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

      await logger.addLog(
        `Профиль создан: ${newProfileName.value} (${newProfileType.value})`,
      );

      newProfileName.value = "";
      newProfilePath.value = "";
      newProfileArgs.value = [];
      newProfileScript.value = "";

      await loadProfiles();
    } catch (e) {
      await logger.addLog(`Ошибка создания профиля: ${e}`);
      console.error("Failed to create profile:", e);
      throw e;
    }
  }

  async function deleteProfile(id: string, name: string) {
    try {
      await invoke("delete_profile", { profileId: id });
      await logger.addLog(`Профиль "${name}" удалён`);
      await loadProfiles();
    } catch (e) {
      await logger.addLog(`Ошибка удаления профиля ${id}: ${e}`);
      console.error("Failed to delete profile:", e);
    }
  }

  function resetProfileForm() {
    newProfileName.value = "";
    newProfilePath.value = "";
    newProfileDescription.value = "";
    newProfileType.value = "";
    newProfileArgs.value = [];
    newProfileScript.value = "";
  }

  return {
    profiles,
    newProfileName,
    newProfilePath,
    newProfileDescription,
    newProfileType,
    newProfileArgs,
    newProfileScript,
    loadProfiles,
    runProfile,
    createProfile,
    deleteProfile,
    resetProfileForm,
  };
}

export type UseProfileManagerReturn = ReturnType<typeof useProfileManager>;
