import type { InstanceEntry } from "@/lib/generated/InstanceEntry";
import type { InstanceSettings } from "@/lib/generated/InstanceSettings";

export type InstanceDraft = {
  overrideWindow: boolean;
  overrideMemory: boolean;
  overrideJava: boolean;
  overrideJvmArguments: boolean;
  javaPath: string;
  maxMemoryMb: number;
  jvmArguments: string;
  fullscreen: boolean;
  width: string;
  height: string;
};

export function fromEntry(entry: InstanceEntry): InstanceDraft {
  return {
    overrideWindow: entry.overrideWindow,
    overrideMemory: entry.overrideMemory,
    overrideJava: entry.overrideJava,
    overrideJvmArguments: entry.overrideJvmArguments,
    javaPath: entry.javaPath ?? "",
    maxMemoryMb: entry.maxMemoryMb,
    jvmArguments: entry.jvmArguments.join(" "),
    fullscreen: entry.fullscreen,
    width: entry.width === null ? "" : String(entry.width),
    height: entry.height === null ? "" : String(entry.height),
  };
}

export function optionalSize(value: string): number | null | "invalid" {
  const trimmed = value.trim();
  if (trimmed === "") {
    return null;
  }
  const parsed = Number(trimmed);
  if (!Number.isInteger(parsed) || parsed < 1) {
    return "invalid";
  }
  return parsed;
}

export function toInstanceSettings(
  draft: InstanceDraft,
  width: number | null,
  height: number | null,
): InstanceSettings {
  const jvmArguments = draft.jvmArguments
    .trim()
    .split(/\s+/)
    .filter((part) => part.length > 0);
  return {
    overrideWindow: draft.overrideWindow,
    overrideMemory: draft.overrideMemory,
    overrideJava: draft.overrideJava,
    overrideJvmArguments: draft.overrideJvmArguments,
    javaPath: draft.javaPath.trim() === "" ? null : draft.javaPath.trim(),
    maxMemoryMb: draft.maxMemoryMb,
    jvmArguments,
    fullscreen: draft.fullscreen,
    width,
    height,
  };
}
