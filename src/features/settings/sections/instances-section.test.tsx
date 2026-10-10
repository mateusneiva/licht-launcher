import { fireEvent, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  mockSettingsCommands,
  renderSettings,
} from "@/features/settings/fixtures/render-settings";

vi.mock("@tauri-apps/api/core", async () => {
  const { invoke: mocked } = await import(
    "@/features/settings/fixtures/invoke"
  );
  return { invoke: mocked };
});

describe("InstancesSection", () => {
  beforeEach(() => {
    mockSettingsCommands();
  });

  it("disables width and height while full screen is on", async () => {
    await renderSettings();
    fireEvent.click(await screen.findByRole("button", { name: "Instances" }));

    const width = screen.getByLabelText("Width");
    const height = screen.getByLabelText("Height");
    expect(width).toBeEnabled();
    expect(height).toBeEnabled();

    fireEvent.click(screen.getByRole("switch", { name: "Full screen" }));

    expect(width).toBeDisabled();
    expect(height).toBeDisabled();
    expect(width).toHaveAttribute("placeholder", "854");
    expect(height).toHaveAttribute("placeholder", "480");
    expect(screen.getByText("Applied to every instance.")).toBeInTheDocument();
    expect(
      screen.getByText(
        "The game starts in full screen. Width and height are not used.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("Window width in pixels.")).toBeInTheDocument();
    expect(screen.getByText("Window height in pixels.")).toBeInTheDocument();
    expect(
      screen.getByText("Maximum memory. The minimum stays 512 MB."),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Java arguments")).toHaveAttribute(
      "placeholder",
      "-XX:+UseG1GC",
    );
    expect(
      screen.getByText("Extra JVM flags, separated by spaces."),
    ).toBeInTheDocument();
  });
});
