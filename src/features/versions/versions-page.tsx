import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useVirtualizer } from "@tanstack/react-virtual";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { installVersion, launchVersion, listVersions } from "@/lib/commands";
import { friendlyError, messageOf } from "@/lib/errors";
import type { DownloadProgress } from "@/lib/generated/DownloadProgress";
import type { GameLine } from "@/lib/generated/GameLine";

const FILTERS = [
  { id: "release", label: "Release" },
  { id: "snapshot", label: "Snapshot" },
  { id: "old_beta", label: "Old beta" },
  { id: "old_alpha", label: "Old alpha" },
  { id: "all", label: "All" },
] as const;

type FilterId = (typeof FILTERS)[number]["id"];

const ROW_HEIGHT = 40;

function percent(progress: DownloadProgress): number {
  if (progress.bytesTotal === 0) {
    return 0;
  }
  return Math.min(100, (progress.bytesDone / progress.bytesTotal) * 100);
}

export function VersionsPage() {
  const queryClient = useQueryClient();
  const [username, setUsername] = useState("");
  const [filter, setFilter] = useState<FilterId>("release");
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [lines, setLines] = useState<Array<GameLine & { id: number }>>([]);
  const nextLine = useRef(0);
  const listParent = useRef<HTMLDivElement>(null);

  const versions = useQuery({
    queryKey: ["versions"],
    queryFn: listVersions,
    retry: false,
  });

  const install = useMutation({
    mutationFn: installVersion,
    onSuccess: async () => {
      setProgress(null);
      await queryClient.invalidateQueries({ queryKey: ["versions"] });
    },
    onError: (error: unknown) => {
      setProgress(null);
      toast.error(friendlyError(messageOf(error)));
    },
  });

  const play = useMutation({
    mutationFn: (versionId: string) =>
      launchVersion(versionId, username.trim()),
    onMutate: () => {
      setLines([]);
    },
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  useEffect(() => {
    if (versions.error) {
      toast.error(friendlyError(messageOf(versions.error)));
    }
  }, [versions.error]);

  useEffect(() => {
    let stop = false;
    let unlistenProgress: (() => void) | undefined;
    let unlistenLog: (() => void) | undefined;
    void listen<DownloadProgress>("install-progress", (event) => {
      setProgress(event.payload);
    }).then((unlisten) => {
      if (stop) {
        unlisten();
      } else {
        unlistenProgress = unlisten;
      }
    });
    void listen<GameLine>("game-log", (event) => {
      const id = nextLine.current;
      nextLine.current += 1;
      setLines((current) => [...current, { ...event.payload, id }]);
    }).then((unlisten) => {
      if (stop) {
        unlisten();
      } else {
        unlistenLog = unlisten;
      }
    });
    return () => {
      stop = true;
      unlistenProgress?.();
      unlistenLog?.();
    };
  }, []);

  const rows = (versions.data ?? []).filter((version) =>
    filter === "all" ? true : version.versionType === filter,
  );
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => listParent.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });
  const busy = install.isPending || play.isPending;
  const nameMissing = username.trim() === "";

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-4 p-6">
      <div className="flex items-center justify-between gap-3">
        <h1 className="text-xl font-semibold">Licht Launcher</h1>
        <Button variant="link" asChild>
          <Link to="/about">About</Link>
        </Button>
      </div>
      <label className="flex flex-col gap-1 text-sm" htmlFor="username">
        Username
        <Input
          id="username"
          value={username}
          autoComplete="nickname"
          onChange={(event) => {
            setUsername(event.target.value);
          }}
        />
      </label>
      <Tabs
        value={filter}
        onValueChange={(value) => {
          setFilter(value as FilterId);
        }}
      >
        <TabsList>
          {FILTERS.map((item) => (
            <TabsTrigger key={item.id} value={item.id}>
              {item.label}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
      {versions.isPending ? (
        <p className="text-sm text-muted-foreground">Loading versions...</p>
      ) : null}
      {versions.isError ? (
        <p className="text-sm text-destructive">
          Versions could not be loaded.
        </p>
      ) : null}
      <div ref={listParent} className="h-96 overflow-auto rounded-lg border">
        {rows.length === 0 && versions.isSuccess ? (
          <p className="p-3 text-sm text-muted-foreground">
            No versions in this group.
          </p>
        ) : (
          <ul
            aria-label="Versions"
            className="relative w-full"
            style={{ height: virtualizer.getTotalSize() }}
          >
            {virtualizer.getVirtualItems().map((item) => {
              const version = rows[item.index];
              if (!version) {
                return null;
              }
              return (
                <li
                  key={version.id}
                  className="absolute inset-x-0 flex items-center justify-between gap-3 px-3"
                  style={{
                    height: ROW_HEIGHT,
                    transform: `translateY(${item.start}px)`,
                  }}
                >
                  <span className="truncate text-sm">{version.id}</span>
                  {version.installed ? (
                    <Button
                      size="sm"
                      disabled={busy || nameMissing}
                      onClick={() => {
                        play.mutate(version.id);
                      }}
                    >
                      Play {version.id}
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={busy}
                      onClick={() => {
                        install.mutate(version.id);
                      }}
                    >
                      Install {version.id}
                    </Button>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
      {install.isPending ? (
        <div className="flex flex-col gap-1">
          <p className="text-sm text-muted-foreground">
            Installing {install.variables}
          </p>
          <Progress value={progress ? percent(progress) : 0} />
        </div>
      ) : null}
      <section aria-label="Game log" className="flex flex-col gap-1">
        <h2 className="text-sm font-medium">Game log</h2>
        <ScrollArea className="h-48 rounded-lg border">
          {lines.length === 0 ? (
            <p className="p-3 text-sm text-muted-foreground">
              The game log will appear here.
            </p>
          ) : (
            <pre className="p-3 text-sm">
              {lines.map((line) => (
                <div
                  key={line.id}
                  className={
                    line.stream === "stderr" ? "text-destructive" : undefined
                  }
                >
                  {line.line}
                </div>
              ))}
            </pre>
          )}
        </ScrollArea>
      </section>
    </main>
  );
}
