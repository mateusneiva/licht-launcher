import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useVirtualizer } from "@tanstack/react-virtual";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  createInstance,
  deleteInstance,
  duplicateInstance,
  installVersion,
  launchVersion,
  listInstances,
  listVersions,
  renameInstance,
} from "@/lib/commands";
import { friendlyError, messageOf } from "@/lib/errors";
import type { DownloadProgress } from "@/lib/generated/DownloadProgress";
import type { GameLine } from "@/lib/generated/GameLine";
import type { VersionSummary } from "@/lib/generated/VersionSummary";

const VIEWS = [
  { id: "versions", label: "Versions" },
  { id: "console", label: "Console" },
] as const;

const TYPES = [
  { id: "release", label: "Release" },
  { id: "snapshot", label: "Snapshot" },
  { id: "old_beta", label: "Old beta" },
  { id: "old_alpha", label: "Old alpha" },
  { id: "all", label: "All" },
] as const;

type ViewId = (typeof VIEWS)[number]["id"];

type InstanceAction =
  | { kind: "create" }
  | { kind: "rename"; folder: string; name: string }
  | { kind: "duplicate"; folder: string; name: string }
  | { kind: "delete"; folder: string; name: string };

type TypeId = (typeof TYPES)[number]["id"];

type ListRow =
  | { kind: "heading"; id: "installed" | "available"; label: string }
  | { kind: "version"; version: VersionSummary };

function listRows(versions: VersionSummary[], typeId: TypeId): ListRow[] {
  const matched = versions.filter((version) =>
    typeId === "all" ? true : version.versionType === typeId,
  );
  const installed = matched.filter((version) => version.installed);
  const available = matched.filter((version) => !version.installed);
  const rows: ListRow[] = [];
  if (installed.length > 0) {
    rows.push({ kind: "heading", id: "installed", label: "Installed" });
    for (const version of installed) {
      rows.push({ kind: "version", version });
    }
  }
  if (available.length > 0) {
    rows.push({ kind: "heading", id: "available", label: "Available" });
    for (const version of available) {
      rows.push({ kind: "version", version });
    }
  }
  return rows;
}

const ROW_HEIGHT = 40;

function ratio(done: number, total: number): number {
  if (total === 0) {
    return 0;
  }
  return Math.min(100, (done / total) * 100);
}

function barValue(progress: DownloadProgress): number {
  if (progress.bytesTotal === 0) {
    return ratio(progress.finished, progress.total);
  }
  return ratio(progress.bytesDone, progress.bytesTotal);
}

function oneDecimal(whole: number, fraction: number): string {
  return `${Math.floor(whole)}.${fraction}`;
}

function megabytes(bytes: number): string {
  const tenths = Math.floor((bytes * 10) / 1_000_000);
  return oneDecimal(tenths / 10, tenths % 10);
}

function progressText(progress: DownloadProgress): string {
  const tenths =
    progress.bytesTotal === 0
      ? 0
      : Math.floor((progress.bytesDone * 1000) / progress.bytesTotal);
  return `${oneDecimal(tenths / 10, tenths % 10)}%  ${megabytes(progress.bytesDone)}MB/${megabytes(progress.bytesTotal)}MB`;
}

export function VersionsPage() {
  const queryClient = useQueryClient();
  const [username, setUsername] = useState("Steve");
  const [view, setView] = useState<ViewId>("versions");
  const [typeId, setTypeId] = useState<TypeId>("release");
  const [action, setAction] = useState<InstanceAction | null>(null);
  const [draftName, setDraftName] = useState("");
  const [draftVersion, setDraftVersion] = useState("");
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [lines, setLines] = useState<Array<GameLine & { id: number }>>([]);
  const nextLine = useRef(0);
  const listParent = useRef<HTMLDivElement>(null);

  const versions = useQuery({
    queryKey: ["versions"],
    queryFn: listVersions,
    retry: false,
  });

  const instances = useQuery({
    queryKey: ["instances"],
    queryFn: listInstances,
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

  const refreshInstances = async () => {
    setAction(null);
    await queryClient.invalidateQueries({ queryKey: ["instances"] });
  };

  const create = useMutation({
    mutationFn: (input: { name: string; versionId: string }) =>
      createInstance(input.name, input.versionId),
    onSuccess: refreshInstances,
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  const rename = useMutation({
    mutationFn: (input: { folder: string; name: string }) =>
      renameInstance(input.folder, input.name),
    onSuccess: refreshInstances,
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  const duplicate = useMutation({
    mutationFn: (input: { folder: string; name: string }) =>
      duplicateInstance(input.folder, input.name),
    onSuccess: refreshInstances,
    onError: (error: unknown) => {
      toast.error(friendlyError(messageOf(error)));
    },
  });

  const remove = useMutation({
    mutationFn: deleteInstance,
    onSuccess: refreshInstances,
    onError: (error: unknown) => {
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
    if (instances.error) {
      toast.error(friendlyError(messageOf(instances.error)));
    }
  }, [instances.error]);

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

  const rows = listRows(versions.data ?? [], typeId);
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => listParent.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
  });
  const busy =
    install.isPending ||
    play.isPending ||
    create.isPending ||
    rename.isPending ||
    duplicate.isPending ||
    remove.isPending;
  const nameMissing = username.trim() === "";
  const installedVersions = (versions.data ?? []).filter(
    (version) => version.installed,
  );

  function openCreate() {
    setDraftName("");
    setDraftVersion(installedVersions[0]?.id ?? "");
    setAction({ kind: "create" });
  }

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-4 p-6">
      <div className="flex flex-col gap-1">
        <div className="flex items-baseline justify-between gap-3">
          <h1 className="text-xl font-semibold">Licht Launcher</h1>
          <div className="flex items-baseline gap-3">
            <Link
              to="/settings"
              className="text-sm text-primary underline-offset-4 hover:underline"
            >
              Settings
            </Link>
            <p className="text-xs text-muted-foreground">0.1.0 · Lilie</p>
          </div>
        </div>
        <p className="text-xs text-muted-foreground">
          Unofficial, not affiliated with Mojang Studios or Microsoft.
        </p>
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
        value={view}
        onValueChange={(value) => {
          setView(value as ViewId);
        }}
      >
        <TabsList>
          {VIEWS.map((item) => (
            <TabsTrigger key={item.id} value={item.id}>
              {item.label}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
      {view === "versions" ? (
        <section aria-label="Instances" className="flex flex-col gap-2">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-sm font-medium">Instances</h2>
            <Button size="sm" disabled={busy} onClick={openCreate}>
              Create
            </Button>
          </div>
          {instances.isPending ? (
            <p className="text-sm text-muted-foreground">
              Loading instances...
            </p>
          ) : null}
          {instances.isSuccess && instances.data.length === 0 ? (
            <p className="text-sm text-muted-foreground">No instances yet.</p>
          ) : null}
          {instances.isSuccess && instances.data.length > 0 ? (
            <ul className="flex flex-col gap-2">
              {instances.data.map((instance) => (
                <li
                  key={instance.folder}
                  className="flex items-center justify-between gap-3"
                >
                  <span className="min-w-0 truncate text-sm">
                    {instance.name}{" "}
                    <span className="text-muted-foreground">
                      {instance.versionId}
                    </span>
                  </span>
                  <span className="flex shrink-0 gap-2">
                    <Button
                      size="sm"
                      disabled={busy || nameMissing}
                      onClick={() => {
                        play.mutate(instance.versionId);
                      }}
                    >
                      Play {instance.name}
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={busy}
                      onClick={() => {
                        setDraftName(instance.name);
                        setAction({
                          kind: "rename",
                          folder: instance.folder,
                          name: instance.name,
                        });
                      }}
                    >
                      Rename {instance.name}
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={busy}
                      onClick={() => {
                        setDraftName("");
                        setAction({
                          kind: "duplicate",
                          folder: instance.folder,
                          name: instance.name,
                        });
                      }}
                    >
                      Duplicate {instance.name}
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      disabled={busy}
                      onClick={() => {
                        setAction({
                          kind: "delete",
                          folder: instance.folder,
                          name: instance.name,
                        });
                      }}
                    >
                      Delete {instance.name}
                    </Button>
                  </span>
                </li>
              ))}
            </ul>
          ) : null}
        </section>
      ) : null}
      <Dialog
        open={action !== null}
        onOpenChange={(open) => {
          if (!open) {
            setAction(null);
          }
        }}
      >
        <DialogContent>
          {action?.kind === "create" ? (
            <>
              <DialogHeader>
                <DialogTitle>Create instance</DialogTitle>
                <DialogDescription>
                  Choose a name and an installed version.
                </DialogDescription>
              </DialogHeader>
              <label
                className="flex flex-col gap-1 text-sm"
                htmlFor="instance-name"
              >
                Name
                <Input
                  id="instance-name"
                  value={draftName}
                  onChange={(event) => {
                    setDraftName(event.target.value);
                  }}
                />
              </label>
              {installedVersions.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  Install a version first.
                </p>
              ) : (
                <Select
                  value={draftVersion}
                  onValueChange={(value) => {
                    setDraftVersion(value);
                  }}
                >
                  <SelectTrigger aria-label="Version">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {installedVersions.map((version) => (
                      <SelectItem key={version.id} value={version.id}>
                        {version.id}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
              <DialogFooter>
                <Button
                  disabled={
                    busy || draftName.trim() === "" || draftVersion === ""
                  }
                  onClick={() => {
                    create.mutate({ name: draftName, versionId: draftVersion });
                  }}
                >
                  Create instance
                </Button>
              </DialogFooter>
            </>
          ) : null}
          {action?.kind === "rename" || action?.kind === "duplicate" ? (
            <>
              <DialogHeader>
                <DialogTitle>
                  {action.kind === "rename" ? "Rename" : "Duplicate"}{" "}
                  {action.name}
                </DialogTitle>
                <DialogDescription>
                  {action.kind === "rename"
                    ? "The folder stays the same."
                    : "Saves are copied into the new instance."}
                </DialogDescription>
              </DialogHeader>
              <label
                className="flex flex-col gap-1 text-sm"
                htmlFor="instance-name"
              >
                Name
                <Input
                  id="instance-name"
                  value={draftName}
                  onChange={(event) => {
                    setDraftName(event.target.value);
                  }}
                />
              </label>
              <DialogFooter>
                <Button
                  disabled={busy || draftName.trim() === ""}
                  onClick={() => {
                    if (action.kind === "rename") {
                      rename.mutate({ folder: action.folder, name: draftName });
                    } else {
                      duplicate.mutate({
                        folder: action.folder,
                        name: draftName,
                      });
                    }
                  }}
                >
                  {action.kind === "rename"
                    ? "Save name"
                    : "Duplicate instance"}
                </Button>
              </DialogFooter>
            </>
          ) : null}
          {action?.kind === "delete" ? (
            <>
              <DialogHeader>
                <DialogTitle>Delete {action.name}?</DialogTitle>
                <DialogDescription>
                  Saves in this instance will be removed.
                </DialogDescription>
              </DialogHeader>
              <DialogFooter>
                <Button
                  variant="destructive"
                  disabled={busy}
                  onClick={() => {
                    remove.mutate(action.folder);
                  }}
                >
                  Delete instance
                </Button>
              </DialogFooter>
            </>
          ) : null}
        </DialogContent>
      </Dialog>
      {view === "console" ? (
        <section aria-label="Game log" className="flex flex-col gap-1">
          <h2 className="text-sm font-medium">Game log</h2>
          <ScrollArea className="h-96 rounded-lg border">
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
      ) : null}
      {view === "versions" ? (
        <Select
          value={typeId}
          onValueChange={(value) => {
            setTypeId(value as TypeId);
          }}
        >
          <SelectTrigger aria-label="Type">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {TYPES.map((item) => (
              <SelectItem key={item.id} value={item.id}>
                {item.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      ) : null}
      {view === "versions" && versions.isPending ? (
        <p className="text-sm text-muted-foreground">Loading versions...</p>
      ) : null}
      {view === "versions" && versions.isError ? (
        <p className="text-sm text-destructive">
          Versions could not be loaded.
        </p>
      ) : null}
      {view === "versions" ? (
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
                const row = rows[item.index];
                if (!row) {
                  return null;
                }
                const position = {
                  height: ROW_HEIGHT,
                  transform: `translateY(${item.start}px)`,
                };
                if (row.kind === "heading") {
                  return (
                    <li
                      key={row.id}
                      className="absolute inset-x-0 flex items-center px-3"
                      style={position}
                    >
                      <h2 className="text-xs font-medium text-muted-foreground">
                        {row.label}
                      </h2>
                    </li>
                  );
                }
                const version = row.version;
                return (
                  <li
                    key={version.id}
                    className="absolute inset-x-0 flex items-center justify-between gap-3 px-3"
                    style={position}
                  >
                    <span className="truncate text-sm">{version.id}</span>
                    {install.isPending && install.variables === version.id ? (
                      <div className="flex min-w-0 flex-1 flex-col justify-center gap-0.5">
                        <span className="truncate text-xs text-muted-foreground">
                          {progress
                            ? progressText(progress)
                            : "0.0%  0.0MB/0.0MB"}
                        </span>
                        <Progress
                          aria-label="Install progress"
                          value={progress ? barValue(progress) : 0}
                        />
                      </div>
                    ) : null}
                    {version.installed ? null : (
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
      ) : null}
    </main>
  );
}
