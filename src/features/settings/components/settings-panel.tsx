import type { Draft } from "@/features/settings/model/draft";
import { SECTIONS, type SectionId } from "@/features/settings/model/sections";
import { AppearanceSection } from "@/features/settings/sections/appearance-section";
import { DownloadsSection } from "@/features/settings/sections/downloads-section";
import { InstancesSection } from "@/features/settings/sections/instances-section";
import { JavaSection } from "@/features/settings/sections/java/java-section";
import type { SettingsSnapshot } from "@/lib/generated/SettingsSnapshot";

export function SettingsPanel({
  section,
  draft,
  snapshot,
  onChange,
}: {
  section: SectionId;
  draft: Draft;
  snapshot: SettingsSnapshot;
  onChange: (patch: Partial<Draft>) => void;
}) {
  const page = SECTIONS.find((item) => item.id === section) ?? SECTIONS[0];
  const PageIcon = page.icon;

  return (
    <div className="flex min-h-0 min-w-0 flex-1 flex-col gap-4 overflow-y-auto bg-muted/30 p-6">
      <div className="flex flex-col gap-1">
        <h2 className="flex items-center gap-2 text-lg font-medium">
          <PageIcon aria-hidden className="size-5" />
          {page.label}
        </h2>
        <p className="text-sm text-muted-foreground">{page.description}</p>
      </div>
      {section === "appearance" ? (
        <AppearanceSection draft={draft} onChange={onChange} />
      ) : null}
      {section === "instances" ? (
        <InstancesSection
          draft={draft}
          snapshot={snapshot}
          onChange={onChange}
        />
      ) : null}
      {section === "java" ? (
        <JavaSection draft={draft} onChange={onChange} />
      ) : null}
      {section === "downloads" ? (
        <DownloadsSection draft={draft} onChange={onChange} />
      ) : null}
    </div>
  );
}
