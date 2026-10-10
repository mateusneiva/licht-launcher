import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Draft } from "@/features/settings/model/draft";

export function AppearanceSection({
  draft,
  onChange,
}: {
  draft: Draft;
  onChange: (patch: Partial<Draft>) => void;
}) {
  return (
    <div className="flex max-w-xs flex-col gap-1 text-sm">
      <span>Color theme</span>
      <Select
        value={draft.theme}
        onValueChange={(theme) => {
          if (theme === "dark") {
            onChange({ theme });
          }
        }}
      >
        <SelectTrigger aria-label="Color theme">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="dark">Dark</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
