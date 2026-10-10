import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import type { Draft } from "@/features/settings/model/draft";
import type { SettingsSnapshot } from "@/lib/generated/SettingsSnapshot";

export function InstancesSection({
  draft,
  snapshot,
  onChange,
}: {
  draft: Draft;
  snapshot: SettingsSnapshot;
  onChange: (patch: Partial<Draft>) => void;
}) {
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <div className="flex items-center justify-between gap-3">
          <label htmlFor="fullscreen" className="text-sm">
            Full screen
          </label>
          <Switch
            id="fullscreen"
            aria-label="Full screen"
            aria-describedby="fullscreen-hint"
            checked={draft.fullscreen}
            onCheckedChange={(fullscreen) => onChange({ fullscreen })}
          />
        </div>
        <p id="fullscreen-hint" className="text-sm text-muted-foreground">
          The game starts in full screen. Width and height are not used.
        </p>
      </div>
      <div className="flex max-w-xs flex-col gap-1">
        <label className="flex flex-col gap-1 text-sm" htmlFor="width">
          Width
          <Input
            id="width"
            type="number"
            min={1}
            inputMode="numeric"
            placeholder="854"
            aria-describedby="width-hint"
            value={draft.width}
            disabled={draft.fullscreen}
            onChange={(event) => onChange({ width: event.target.value })}
          />
        </label>
        <p id="width-hint" className="text-sm text-muted-foreground">
          Window width in pixels.
        </p>
      </div>
      <div className="flex max-w-xs flex-col gap-1">
        <label className="flex flex-col gap-1 text-sm" htmlFor="height">
          Height
          <Input
            id="height"
            type="number"
            min={1}
            inputMode="numeric"
            placeholder="480"
            aria-describedby="height-hint"
            value={draft.height}
            disabled={draft.fullscreen}
            onChange={(event) => onChange({ height: event.target.value })}
          />
        </label>
        <p id="height-hint" className="text-sm text-muted-foreground">
          Window height in pixels.
        </p>
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex items-baseline justify-between gap-3 text-sm">
          <span>Memory</span>
          <span className="text-muted-foreground">{draft.maxMemoryMb} MB</span>
        </div>
        <Slider
          aria-label="Memory"
          aria-describedby="memory-hint"
          min={snapshot.memoryMinimumMb}
          max={snapshot.memoryMaximumMb}
          step={snapshot.memoryStepMb}
          value={[draft.maxMemoryMb]}
          onValueChange={(value) => {
            const next = value[0];
            if (next !== undefined) {
              onChange({ maxMemoryMb: next });
            }
          }}
        />
        <p id="memory-hint" className="text-sm text-muted-foreground">
          Maximum memory. The minimum stays 512 MB.
        </p>
      </div>
      <div className="flex flex-col gap-1">
        <label className="flex flex-col gap-1 text-sm" htmlFor="jvm-arguments">
          Java arguments
          <Input
            id="jvm-arguments"
            placeholder="-XX:+UseG1GC"
            aria-describedby="jvm-arguments-hint"
            value={draft.jvmArguments}
            onChange={(event) => onChange({ jvmArguments: event.target.value })}
          />
        </label>
        <p id="jvm-arguments-hint" className="text-sm text-muted-foreground">
          Extra JVM flags, separated by spaces.
        </p>
      </div>
    </div>
  );
}
