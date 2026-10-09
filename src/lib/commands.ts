import { invoke } from "@tauri-apps/api/core";

import type { GameExit } from "@/lib/generated/GameExit";
import type { VersionSummary } from "@/lib/generated/VersionSummary";

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
