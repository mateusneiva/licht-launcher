import { describe, expect, it } from "vitest";

import {
  mergeInstallations,
  samePath,
} from "@/features/settings/sections/java/merge-installations";

describe("mergeInstallations", () => {
  it("puts the runtime path first when it is missing from the list", () => {
    expect(
      mergeInstallations(["C:\\Other\\java.exe"], "C:\\Runtime\\java.exe"),
    ).toEqual(["C:\\Runtime\\java.exe", "C:\\Other\\java.exe"]);
  });

  it("does not duplicate the runtime path", () => {
    expect(
      mergeInstallations(["C:\\Runtime\\java.exe"], "C:\\runtime\\java.exe"),
    ).toEqual(["C:\\Runtime\\java.exe"]);
  });
});

describe("samePath", () => {
  it("ignores case and surrounding spaces", () => {
    expect(
      samePath(" C:\\Java\\bin\\java.exe ", "c:\\java\\bin\\java.exe"),
    ).toBe(true);
  });
});
