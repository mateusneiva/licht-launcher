import { describe, expect, it } from "vitest";

import { formatMemoryPreset, memoryPresetOptions } from "@/lib/memory-presets";

describe("memoryPresetOptions", () => {
  it("exposes the ladder up to an 8 GB machine", () => {
    expect(
      memoryPresetOptions({
        minimumMb: 512,
        maximumMb: 8192,
        recommendedMb: 4096,
        currentMb: 2048,
        totalMb: 8192,
      }),
    ).toEqual([2048, 4096, 6144, 8192]);
  });

  it("stops at the machine ceiling on 4 GB", () => {
    expect(
      memoryPresetOptions({
        minimumMb: 512,
        maximumMb: 4096,
        recommendedMb: 2048,
        currentMb: 2048,
        totalMb: 4096,
      }),
    ).toEqual([2048, 4096]);
  });

  it("exposes more stops on 16 GB machines", () => {
    expect(
      memoryPresetOptions({
        minimumMb: 512,
        maximumMb: 16384,
        recommendedMb: 4096,
        currentMb: 4096,
        totalMb: 16384,
      }),
    ).toEqual([2048, 4096, 6144, 8192, 10240, 12288, 16384]);
  });

  it("includes the current value when it is not on the ladder", () => {
    expect(
      memoryPresetOptions({
        minimumMb: 512,
        maximumMb: 8192,
        recommendedMb: 4096,
        currentMb: 1536,
        totalMb: 8192,
      }),
    ).toEqual([1536, 2048, 4096, 6144, 8192]);
  });
});

describe("formatMemoryPreset", () => {
  it("formats whole gigabytes and leftover megabytes", () => {
    expect(formatMemoryPreset(2048)).toBe("2GB");
    expect(formatMemoryPreset(1536)).toBe("1536MB");
  });
});
