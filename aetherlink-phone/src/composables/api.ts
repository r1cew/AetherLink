// api.ts
import { invoke } from "@tauri-apps/api/core";
import type { Server, Profile, DevStatus } from "./types";

export async function getServers(): Promise<Server[]> {
  return await invoke<Server[]>("get_servers");
}

export async function listProfiles(serverId: string): Promise<Profile[]> {
  return await invoke<Profile[]>("list_profiles", { serverId });
}

export async function pairWithQR(
  qrJson: string,
  name: string,
  nickname: string,
): Promise<string> {
  return await invoke<string>("pair_with_qr", { qrJson, name, nickname });
}

export async function checkDevStatus(serverId: string): Promise<DevStatus> {
  return await invoke<DevStatus>("check_dev_status", { serverId });
}

export async function sendSafe(
  serverId: string,
  command: string,
  params?: object,
) {
  return await invoke("send_safe", {
    serverId,
    command,
    params: params ?? null,
  });
}

export async function runProfile(serverId: string, profileId: string) {
  return await invoke("send_run_profile", { serverId, profileId });
}

export async function createProfile(
  serverId: string,
  name: string,
  description: string | undefined,
  commands: any,
) {
  return await invoke("create_profile", {
    serverId,
    name,
    description,
    commands,
  });
}

export async function discoverAndUpdate(serverId: string): Promise<string> {
  return await invoke<string>("discover_and_update", { serverId });
}

export async function removeServer(serverId: string) {
  return await invoke("remove_server", { serverId });
}
