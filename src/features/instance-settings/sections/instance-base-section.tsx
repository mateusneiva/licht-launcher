import { BoxIcon, LayersIcon, PencilIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { EditInstanceVersionDialog } from "@/features/instance-settings/components/edit-instance-version-dialog";
import type { VersionSummary } from "@/lib/generated/VersionSummary";

export function InstanceBaseSection({
  versionId,
  versions,
  busy,
  versionBusy,
  onChangeVersion,
}: {
  versionId: string;
  versions: VersionSummary[];
  busy: boolean;
  versionBusy: boolean;
  onChangeVersion: (versionId: string) => void;
}) {
  const [editOpen, setEditOpen] = useState(false);

  return (
    <div className="flex flex-col gap-6">
      <div className="grid gap-4 rounded-xl bg-background/60 p-4 sm:grid-cols-2">
        <div className="flex min-w-0 flex-col gap-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <LayersIcon aria-hidden className="size-4 shrink-0" />
            <span className="font-medium text-foreground">Platform</span>
          </div>
          <div className="flex h-8 items-center rounded-lg bg-muted/40 px-2.5 text-sm font-medium">
            Vanilla
          </div>
        </div>
        <div className="flex min-w-0 flex-col gap-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <BoxIcon aria-hidden className="size-4 shrink-0" />
            <span className="font-medium text-foreground">Game version</span>
          </div>
          <div className="flex h-8 items-center rounded-lg bg-muted/40 px-2.5 text-sm font-medium">
            {versionBusy ? (
              <span className="flex items-center gap-2">
                <Spinner className="size-3.5" aria-hidden />
                Updating…
              </span>
            ) : (
              <span className="truncate">{versionId}</span>
            )}
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <div className="flex flex-col gap-1">
          <h3 className="text-sm font-medium">Edit base</h3>
          <p className="text-sm text-muted-foreground">
            Open the editor to pick another Vanilla release for this instance.
          </p>
        </div>
        <Button
          type="button"
          variant="warning"
          className="self-start"
          disabled={busy || versionBusy || versions.length === 0}
          onClick={() => {
            setEditOpen(true);
          }}
        >
          <PencilIcon data-icon="inline-start" aria-hidden />
          Edit base
        </Button>
      </div>

      <EditInstanceVersionDialog
        open={editOpen}
        versionId={versionId}
        versions={versions}
        busy={busy || versionBusy}
        onOpenChange={setEditOpen}
        onSave={onChangeVersion}
      />
    </div>
  );
}
