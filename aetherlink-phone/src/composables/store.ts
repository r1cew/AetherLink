// store.ts
import { Store } from "@tauri-apps/plugin-store";
import type { Server, Profile } from "./types";

const storePromise = Store.load("settings.json");

export async function saveConnectionData(
  active: Server | null,
  profiles: Profile[],
) {
  try {
    const store = await storePromise;
    if (active) await store.set("activeServer", active);
    if (profiles.length > 0) await store.set("profiles", profiles);
    await store.save();
  } catch (e) {
    console.error("Ошибка сохранения:", e);
  }
}

export async function loadConnectionData(): Promise<{
  activeServer: Server | null;
  profiles: Profile[];
}> {
  try {
    const store = await storePromise;
    const savedServer = await store.get("activeServer");
    const savedProfiles = await store.get("profiles");

    return {
      activeServer: savedServer as Server | null,
      profiles: (savedProfiles as Profile[]) || [],
    };
  } catch (e) {
    console.error("Ошибка загрузки:", e);
    return { activeServer: null, profiles: [] };
  }
}

export async function resetConnectionData() {
  try {
    const store = await storePromise;
    await store.delete("activeServer");
    await store.delete("profiles");
    await store.save();
  } catch (e) {
    console.error("Ошибка сброса:", e);
  }
}
