import { useMutation } from "@tanstack/react-query";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import type { Draft } from "@/features/settings/model/draft";
import { browseApplicationDirectory } from "@/lib/commands";
import { friendlyError, messageOf } from "@/lib/errors";

export function DownloadsSection({
  draft,
  onChange,
}: {
  draft: Draft;
  onChange: (patch: Partial<Draft>) => void;
}) {
  const browseDirectory = useMutation({
    mutationFn: browseApplicationDirectory,
    onSuccess: (path) => {
      if (path) {
        onChange({ dataDirectory: path });
      }
    },
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2">
        <div className="flex items-baseline justify-between gap-3 text-sm">
          <span id="download-concurrency-label">Download concurrency</span>
          <span className="text-muted-foreground">
            {draft.downloadConcurrency}
          </span>
        </div>
        <Slider
          aria-labelledby="download-concurrency-label"
          min={1}
          max={16}
          step={1}
          value={[draft.downloadConcurrency]}
          onValueChange={(value) => {
            const next = value[0];
            if (next !== undefined) {
              onChange({ downloadConcurrency: next });
            }
          }}
        />
      </div>
      <div className="flex flex-col gap-1 text-sm">
        <label htmlFor="application-directory">Application directory</label>
        <div className="flex gap-2">
          <Input
            id="application-directory"
            className="min-w-0 flex-1"
            value={draft.dataDirectory}
            onChange={(event) =>
              onChange({ dataDirectory: event.target.value })
            }
          />
          <Button
            type="button"
            variant="secondary"
            className="shrink-0"
            disabled={browseDirectory.isPending}
            onClick={() => browseDirectory.mutate(draft.dataDirectory)}
          >
            Browse
          </Button>
        </div>
      </div>
      <p className="text-sm text-muted-foreground">
        This change applies after restart.
      </p>
    </div>
  );
}
