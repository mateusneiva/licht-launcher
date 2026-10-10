import { invoke } from "@tauri-apps/api/core";

import type { GameExit } from "@/lib/generated/GameExit";
import type { InstanceEntry } from "@/lib/generated/InstanceEntry";
import type { VersionSummary } from "@/lib/generated/VersionSummary";

export function listInstances(): Promise<InstanceEntry[]> {
  return invoke("list_instances");
}

export function createInstance(
  name: string,
  versionId: string,
): Promise<InstanceEntry> {
  return invoke("create_instance", { name, versionId });
}

export function renameInstance(
  folder: string,
  name: string,
): Promise<InstanceEntry> {
  return invoke("rename_instance", { folder, name });
}

export function duplicateInstance(
  folder: string,
  name: string,
): Promise<InstanceEntry> {
  return invoke("duplicate_instance", { folder, name });
}

export function deleteInstance(folder: string): Promise<void> {
  return invoke("delete_instance", { folder });
}

export function listVersions(): Promise<VersionSummary[]> {
  return invoke("list_versions");
}

export function installVersion(versionId: string): Promise<string> {
  return invoke("install_version", { versionId });
}

export function launchVersion(
  versionId: string,
  username: string,
): Promise<GameExit> {
  return invoke("launch", { versionId, username });
}
