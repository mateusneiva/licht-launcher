import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  getSettings,
  installVersion,
  launchVersion,
  listVersions,
  setDownloadConcurrency,
} from "@/lib/commands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("launcher commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue(undefined);
  });

  it("reads settings without arguments", async () => {
    await getSettings();
    expect(invoke).toHaveBeenCalledWith("get_settings");
  });

  it("saves download concurrency", async () => {
    await setDownloadConcurrency(4);
    expect(invoke).toHaveBeenCalledWith("set_download_concurrency", {
      downloadConcurrency: 4,
    });
  });

  it("lists versions without arguments", async () => {
    await listVersions();
    expect(invoke).toHaveBeenCalledWith("list_versions");
  });

  it("installs one version by id", async () => {
    await installVersion("1.20.1");
    expect(invoke).toHaveBeenCalledWith("install_version", {
      versionId: "1.20.1",
    });
  });

  it("launches with the offline username", async () => {
    await launchVersion("1.5.2", "Mateus");
    expect(invoke).toHaveBeenCalledWith("launch", {
      versionId: "1.5.2",
      username: "Mateus",
    });
  });
});
