import type { CSSProperties } from "react";

import { Slider } from "@/components/ui/slider";
import { formatMemoryPreset, memoryPresetOptions } from "@/lib/memory-presets";

/** Half of `size-4` thumb — matches Radix in-bounds thumb centers. */
const THUMB_HALF_PX = 8;

function markTranslateClass(index: number, count: number): string {
  if (count <= 1 || index === 0) {
    return "translate-x-0";
  }
  if (index === count - 1) {
    return "-translate-x-full";
  }
  return "-translate-x-1/2";
}

function labelLeftStyle(
  index: number,
  count: number,
  percent: number,
): CSSProperties {
  if (count <= 1 || index === 0 || index === count - 1) {
    return { left: `${percent}%` };
  }
  const inset = THUMB_HALF_PX - (percent / 100) * (THUMB_HALF_PX * 2);
  return { left: `calc(${percent}% + ${inset}px)` };
}

function tickLeftStyle(percent: number): CSSProperties {
  const inset = THUMB_HALF_PX - (percent / 100) * (THUMB_HALF_PX * 2);
  return { left: `calc(${percent}% + ${inset}px)` };
}

export function MemorySlider({
  valueMb,
  minimumMb,
  maximumMb,
  recommendedMb,
  totalMb,
  onChange,
}: {
  valueMb: number;
  minimumMb: number;
  maximumMb: number;
  recommendedMb: number;
  totalMb: number;
  onChange: (maxMemoryMb: number) => void;
}) {
  const options = memoryPresetOptions({
    minimumMb,
    maximumMb,
    recommendedMb,
    currentMb: valueMb,
    totalMb,
  });
  const index = Math.max(0, options.indexOf(valueMb));
  const lastIndex = Math.max(options.length - 1, 0);

  return (
    <div className="flex w-full flex-col gap-3">
      <Slider
        aria-label="Memory"
        className="w-full"
        min={0}
        max={lastIndex}
        step={1}
        value={[index]}
        onValueChange={(value) => {
          const nextIndex = value[0];
          if (nextIndex === undefined) {
            return;
          }
          const next = options[nextIndex];
          if (next !== undefined) {
            onChange(next);
          }
        }}
      >
        <div
          aria-hidden
          className="pointer-events-none absolute inset-x-0 top-1/2 z-10 h-0"
        >
          {options.map((mb, optionIndex) => {
            const percent =
              lastIndex === 0 ? 0 : (optionIndex / lastIndex) * 100;
            const selected = mb === valueMb;
            return (
              <span
                key={mb}
                className={
                  selected
                    ? "absolute top-1/2 h-2.5 w-px -translate-x-1/2 -translate-y-1/2 bg-primary-foreground/80"
                    : "absolute top-1/2 h-2.5 w-px -translate-x-1/2 -translate-y-1/2 bg-muted-foreground/50"
                }
                style={tickLeftStyle(percent)}
              />
            );
          })}
        </div>
      </Slider>
      <div className="relative h-8 w-full">
        {options.map((mb, optionIndex) => {
          const selected = mb === valueMb;
          const percent =
            lastIndex === 0 ? 0 : (optionIndex / lastIndex) * 100;
          const isEnd =
            options.length > 1 &&
            (optionIndex === 0 || optionIndex === lastIndex);
          return (
            <button
              key={mb}
              type="button"
              className={
                selected
                  ? `absolute top-0 ${markTranslateClass(optionIndex, options.length)} cursor-pointer rounded-md bg-primary/15 text-[0.8rem] font-medium whitespace-nowrap text-foreground ${isEnd ? "px-2 py-1" : "px-2.5 py-1"}`
                  : `absolute top-0 ${markTranslateClass(optionIndex, options.length)} cursor-pointer rounded-md text-[0.8rem] whitespace-nowrap text-muted-foreground hover:bg-muted hover:text-foreground ${isEnd ? "px-2 py-1" : "px-2.5 py-1"}`
              }
              style={labelLeftStyle(optionIndex, options.length, percent)}
              aria-label={formatMemoryPreset(mb)}
              aria-pressed={selected}
              onClick={() => {
                onChange(mb);
              }}
            >
              {formatMemoryPreset(mb)}
            </button>
          );
        })}
      </div>
    </div>
  );
}
