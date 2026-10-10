import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { MemorySlider } from "@/features/settings/components/memory-slider";
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
    <div className="flex flex-col gap-6">
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 flex-col gap-1">
            <label htmlFor="fullscreen" className="text-sm font-medium">
              Fullscreen
            </label>
            <p id="fullscreen-hint" className="text-sm text-muted-foreground">
              The game starts in fullscreen. Width and height are not used.
            </p>
          </div>
          <Switch
            id="fullscreen"
            className="shrink-0"
            aria-label="Fullscreen"
            aria-describedby="fullscreen-hint"
            checked={draft.fullscreen}
            onCheckedChange={(fullscreen) => onChange({ fullscreen })}
          />
        </div>
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 flex-col gap-1">
            <label htmlFor="width" className="text-sm font-medium">
              Width
            </label>
            <p id="width-hint" className="text-sm text-muted-foreground">
              Window width in pixels.
            </p>
          </div>
          <Input
            id="width"
            type="number"
            min={1}
            inputMode="numeric"
            placeholder="854"
            className="w-28 shrink-0"
            aria-describedby="width-hint"
            value={draft.width}
            disabled={draft.fullscreen}
            onChange={(event) => onChange({ width: event.target.value })}
          />
        </div>
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 flex-col gap-1">
            <label htmlFor="height" className="text-sm font-medium">
              Height
            </label>
            <p id="height-hint" className="text-sm text-muted-foreground">
              Window height in pixels.
            </p>
          </div>
          <Input
            id="height"
            type="number"
            min={1}
            inputMode="numeric"
            placeholder="480"
            className="w-28 shrink-0"
            aria-describedby="height-hint"
            value={draft.height}
            disabled={draft.fullscreen}
            onChange={(event) => onChange({ height: event.target.value })}
          />
        </div>
      </section>

      <section className="flex flex-col gap-4 border-t border-border pt-6">
        <div className="flex flex-col gap-1">
          <div className="flex items-baseline justify-between gap-3">
            <h3 className="text-sm font-medium">Memory</h3>
            <span className="text-sm text-muted-foreground">
              {draft.maxMemoryMb} MB
            </span>
          </div>
          <p id="memory-hint" className="text-sm text-muted-foreground">
            Maximum memory. The minimum stays 512 MB.
          </p>
        </div>
        <MemorySlider
          valueMb={draft.maxMemoryMb}
          minimumMb={snapshot.memoryMinimumMb}
          maximumMb={snapshot.memoryMaximumMb}
          recommendedMb={snapshot.memoryRecommendedMb}
          totalMb={snapshot.memoryTotalMb}
          onChange={(maxMemoryMb) => {
            onChange({ maxMemoryMb });
          }}
        />
      </section>
    </div>
  );
}
