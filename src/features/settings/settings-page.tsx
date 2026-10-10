import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { getSettings, setDownloadConcurrency } from "@/lib/commands";
import { friendlyError, messageOf } from "@/lib/errors";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const settings = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
    retry: false,
  });
  const [draft, setDraft] = useState<string | null>(null);
  const shown =
    draft ??
    (settings.data === undefined
      ? ""
      : String(settings.data.downloadConcurrency));

  const save = useMutation({
    mutationFn: setDownloadConcurrency,
    onSuccess: async () => {
      setDraft(null);
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  useEffect(() => {
    if (settings.error) {
      toast.error(friendlyError(messageOf(settings.error)));
    }
  }, [settings.error]);

  function saveConcurrency() {
    const parsed = Number(shown);
    if (!Number.isInteger(parsed)) {
      toast.error("download concurrency must be from 1 to 16");
      return;
    }
    save.mutate(parsed);
  }

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-4 p-6">
      <div className="flex flex-col gap-1">
        <div className="flex items-baseline justify-between gap-3">
          <h1 className="text-xl font-semibold">Settings</h1>
          <Link
            to="/"
            className="text-sm text-primary underline-offset-4 hover:underline"
          >
            Versions
          </Link>
        </div>
        <p className="text-xs text-muted-foreground">
          Unofficial, not affiliated with Mojang Studios or Microsoft.
        </p>
      </div>
      <label
        className="flex max-w-xs flex-col gap-1 text-sm"
        htmlFor="download-concurrency"
      >
        Download concurrency
        <Input
          id="download-concurrency"
          type="number"
          min={1}
          max={16}
          step={1}
          inputMode="numeric"
          value={shown}
          disabled={settings.data === undefined || save.isPending}
          onChange={(event) => {
            setDraft(event.target.value);
          }}
        />
      </label>
      <p className="text-sm text-muted-foreground">
        How many files download at once. From 1 to 16.
      </p>
      <Button
        className="w-fit"
        disabled={settings.data === undefined || save.isPending}
        onClick={saveConcurrency}
      >
        Save
      </Button>
    </main>
  );
}
