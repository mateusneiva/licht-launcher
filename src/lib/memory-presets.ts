/** Common allocation stops; filtered by the machine maximum. */
const MEMORY_LADDER_MB = [
  2048, 4096, 6144, 8192, 10240, 12288, 16384,
] as const;

/** Discrete memory choices within the machine limits. */
export function memoryPresetOptions(input: {
  minimumMb: number;
  maximumMb: number;
  recommendedMb: number;
  currentMb: number;
  totalMb: number;
}): number[] {
  const ceiling = Math.min(input.maximumMb, input.totalMb);
  const options = new Set<number>();
  for (const mb of MEMORY_LADDER_MB) {
    if (mb >= input.minimumMb && mb <= ceiling) {
      options.add(mb);
    }
  }
  if (
    input.recommendedMb >= input.minimumMb &&
    input.recommendedMb <= ceiling
  ) {
    options.add(input.recommendedMb);
  }
  if (ceiling >= input.minimumMb) {
    options.add(ceiling);
  }
  if (
    input.currentMb >= input.minimumMb &&
    input.currentMb <= ceiling
  ) {
    options.add(input.currentMb);
  }
  return [...options].sort((left, right) => left - right);
}

export function formatMemoryPreset(mb: number): string {
  if (mb % 1024 === 0) {
    return `${mb / 1024}GB`;
  }
  return `${mb}MB`;
}
