import { invoke } from "@tauri-apps/api/core";

import type { GameExit } from "@/lib/generated/GameExit";
import type { Settings } from "@/lib/generated/Settings";
import type { VersionSummary } from "@/lib/generated/VersionSummary";

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function setDownloadConcurrency(
  downloadConcurrency: number,
): Promise<Settings> {
  return invoke("set_download_concurrency", { downloadConcurrency });
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
