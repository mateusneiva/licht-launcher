import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { Toaster } from "@/components/ui/sonner";
import { routeTree } from "@/routeTree.gen";

const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

async function renderSettings() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ["/settings"] }),
  });
  await router.load();
  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <Toaster />
    </QueryClientProvider>,
  );
}

describe("SettingsPage", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockImplementation((command: string) => {
      if (command === "get_settings") {
        return Promise.resolve({ schema: 1, downloadConcurrency: 8 });
      }
      if (command === "set_download_concurrency") {
        return Promise.resolve({ schema: 1, downloadConcurrency: 4 });
      }
      return Promise.resolve(undefined);
    });
  });

  it("shows the default concurrency and saves a new value", async () => {
    await renderSettings();

    const field = await screen.findByLabelText("Download concurrency");
    expect(field).toHaveValue(8);

    fireEvent.change(field, { target: { value: "4" } });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("set_download_concurrency", {
        downloadConcurrency: 4,
      });
    });
  });
});
