import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Toaster } from "@/components/ui/sonner";
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

    expect(await screen.findByText("1.20.1")).toBeInTheDocument();
    expect(screen.getByText("1.5.2")).toBeInTheDocument();
    expect(screen.queryByText("24w14a")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Install 1.20.1" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("install_version", {
        versionId: "1.20.1",
      });
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
    selectTab("Snapshot");
    expect(await screen.findByText("24w14a")).toBeInTheDocument();
    expect(screen.queryByText("1.5.2")).not.toBeInTheDocument();

    selectTab("Release");
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

    expect(await screen.findByText("Sound engine started")).toBeInTheDocument();
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
