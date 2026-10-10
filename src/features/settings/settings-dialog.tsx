import { useState } from "react";

import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { SettingsPanel } from "@/features/settings/components/settings-panel";
import { SettingsSidebar } from "@/features/settings/components/settings-sidebar";
import { useSettingsDraft } from "@/features/settings/hooks/use-settings-draft";
import type { SectionId } from "@/features/settings/model/sections";

export function SettingsDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [section, setSection] = useState<SectionId>("appearance");
  const { draft, snapshot, update, flush } = useSettingsDraft(open);

  function handleOpenChange(next: boolean) {
    if (!next) {
      flush();
    }
    onOpenChange(next);
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="flex h-3/4 max-h-screen gap-0 overflow-hidden p-0 sm:max-w-3xl">
        <SettingsSidebar section={section} onSectionChange={setSection} />
        {draft && snapshot ? (
          <SettingsPanel
            section={section}
            draft={draft}
            snapshot={snapshot}
            onChange={update}
          />
        ) : (
          <div className="flex min-h-0 min-w-0 flex-1 items-center justify-center bg-muted/30 p-6">
            <Spinner
              className="size-5 text-muted-foreground"
              aria-label="Loading settings"
            />
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
