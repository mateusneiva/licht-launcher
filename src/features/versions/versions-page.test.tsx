import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Toaster } from "@/components/ui/sonner";
import type { DownloadProgress } from "@/lib/generated/DownloadProgress";
import type { GameLine } from "@/lib/generated/GameLine";
import { routeTree } from "@/routeTree.gen";

const { invoke, listeners } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(
    async (name: string, handler: (event: { payload: unknown }) => void) => {
      listeners.set(name, handler);
      return () => {
        listeners.delete(name);
      };
    },
  ),
}));

function selectTab(name: string) {
  const tab = screen.getByRole("tab", { name });
  fireEvent.mouseDown(tab);
  fireEvent.click(tab);
}

async function selectType(name: string) {
  fireEvent.click(screen.getByRole("combobox", { name: "Type" }));
  fireEvent.click(await screen.findByRole("option", { name }));
}

async function renderAt(path: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  await router.load();
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <Toaster />
    </QueryClientProvider>,
  );
}

describe("VersionsPage", () => {
  beforeEach(() => {
    listeners.clear();
    invoke.mockReset();
    Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
      configurable: true,
      get: () => 800,
    });
    Object.defineProperty(HTMLElement.prototype, "offsetWidth", {
      configurable: true,
      get: () => 600,
    });
  });

  it("lists releases and installs a version that is not on disk", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "list_versions") {
        return Promise.resolve([
          { id: "1.20.1", versionType: "release", installed: false },
          { id: "1.5.2", versionType: "release", installed: true },
          { id: "24w14a", versionType: "snapshot", installed: false },
        ]);
      }
      if (command === "install_version") {
        return Promise.resolve("java");
      }
      return Promise.resolve({ code: 0 });
    });

    await renderAt("/");

    expect(await screen.findByLabelText("Username")).toHaveValue("Steve");
    expect(
      await screen.findByRole("tab", { name: "Versions" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Console" })).toBeInTheDocument();
    expect(await screen.findByText("1.20.1")).toBeInTheDocument();
    expect(screen.getByText("0.1.0 · Lilie")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Unofficial, not affiliated with Mojang Studios or Microsoft.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "About" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("1.5.2")).toBeInTheDocument();
    expect(screen.queryByText("24w14a")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("tab", { name: "Release" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Installed" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Available" }),
    ).toBeInTheDocument();
    expect(
      screen
        .getByRole("heading", { name: "Installed" })
        .compareDocumentPosition(screen.getByText("1.5.2")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen
        .getByText("1.5.2")
        .compareDocumentPosition(
          screen.getByRole("heading", { name: "Available" }),
        ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen
        .getByRole("heading", { name: "Available" })
        .compareDocumentPosition(screen.getByText("1.20.1")) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen.queryByText("The game log will appear here."),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Install 1.20.1" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("install_version", {
        versionId: "1.20.1",
      });
    });

    selectTab("Console");
    expect(
      screen.getByText("The game log will appear here."),
    ).toBeInTheDocument();
    expect(screen.queryByText("1.5.2")).not.toBeInTheDocument();
  });

  it("shows install progress on the version row until the install finishes", async () => {
    let finishInstall: (java: string) => void = () => {};
    invoke.mockImplementation((command: string) => {
      if (command === "list_versions") {
        return Promise.resolve([
          { id: "1.20.1", versionType: "release", installed: false },
        ]);
      }
      if (command === "install_version") {
        return new Promise<string>((resolve) => {
          finishInstall = resolve;
        });
      }
      return Promise.resolve({ code: 0 });
    });

    await renderAt("/");
    fireEvent.click(
      await screen.findByRole("button", { name: "Install 1.20.1" }),
    );

    await waitFor(() => {
      expect(listeners.has("install-progress")).toBe(true);
    });
    listeners.get("install-progress")?.({
      payload: {
        finished: 1,
        failed: 0,
        total: 4,
        bytesDone: 5_000_000,
        bytesTotal: 10_000_000,
      } satisfies DownloadProgress,
    });

    const row = (await screen.findByText("1.20.1")).closest("li");
    const installButton = screen.getByRole("button", {
      name: "Install 1.20.1",
    });
    expect(installButton).toBeDisabled();
    expect(
      await screen.findByText(/50\.0%\s+5\.0MB\/10\.0MB/),
    ).toBeInTheDocument();
    const bar = screen.getByRole("progressbar", { name: "Install progress" });
    expect(row).toContainElement(bar);
    expect(row).toContainElement(installButton);
    expect(bar).toHaveAttribute("aria-valuenow", "50");

    finishInstall("java");
    await waitFor(() => {
      expect(
        screen.queryByText(/50\.0%\s+5\.0MB\/10\.0MB/),
      ).not.toBeInTheDocument();
    });
  });

  it("shows snapshots on that filter and plays an installed version", async () => {
    invoke.mockImplementation((command: string) => {
      if (command === "list_versions") {
        return Promise.resolve([
          { id: "1.5.2", versionType: "release", installed: true },
          { id: "24w14a", versionType: "snapshot", installed: false },
        ]);
      }
      return new Promise(() => {});
    });

    await renderAt("/");
    await screen.findByText("1.5.2");
    await selectType("Snapshot");
    expect(await screen.findByText("24w14a")).toBeInTheDocument();
    expect(screen.queryByText("1.5.2")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "Installed" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Available" }),
    ).toBeInTheDocument();

    await selectType("Release");
    fireEvent.change(screen.getByLabelText("Username"), {
      target: { value: "Mateus" },
    });
    fireEvent.click(await screen.findByRole("button", { name: "Play 1.5.2" }));

    listeners.get("game-log")?.({
      payload: {
        stream: "stdout",
        line: "Sound engine started",
      } satisfies GameLine,
    });
    expect(screen.queryByText("Sound engine started")).not.toBeInTheDocument();

    selectTab("Console");
    expect(await screen.findByText("Sound engine started")).toBeInTheDocument();
    expect(screen.queryByText("1.5.2")).not.toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("launch", {
      versionId: "1.5.2",
      username: "Mateus",
    });
  });

  it("shows a network toast when the list fails", async () => {
    invoke.mockRejectedValue("HTTP request failed");
    await renderAt("/");
    expect(
      await screen.findByText(
        "The network request failed. Check your connection and try again.",
      ),
    ).toBeInTheDocument();
  });
});
