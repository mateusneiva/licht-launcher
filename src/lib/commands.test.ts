import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  createInstance,
  deleteInstance,
  duplicateInstance,
  installVersion,
  launchVersion,
  listInstances,
  listVersions,
  renameInstance,
} from "@/lib/commands";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("launcher commands", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue(undefined);
  });

  it("lists instances without arguments", async () => {
    await listInstances();
    expect(invoke).toHaveBeenCalledWith("list_instances");
  });

  it("creates, renames, duplicates, and deletes an instance", async () => {
    await createInstance("Survival", "1.20.1");
    expect(invoke).toHaveBeenCalledWith("create_instance", {
      name: "Survival",
      versionId: "1.20.1",
    });
    await renameInstance("Survival", "Classic");
    expect(invoke).toHaveBeenCalledWith("rename_instance", {
      folder: "Survival",
      name: "Classic",
    });
    await duplicateInstance("Survival", "Survival copy");
    expect(invoke).toHaveBeenCalledWith("duplicate_instance", {
      folder: "Survival",
      name: "Survival copy",
    });
    await deleteInstance("Survival");
    expect(invoke).toHaveBeenCalledWith("delete_instance", {
      folder: "Survival",
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
