import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { MemorySlider } from "@/features/settings/components/memory-slider";

describe("MemorySlider", () => {
  it("snaps between marked presets", () => {
    const onChange = vi.fn();
    render(
      <MemorySlider
        valueMb={2048}
        minimumMb={512}
        maximumMb={8192}
        recommendedMb={4096}
        totalMb={8192}
        onChange={onChange}
      />,
    );

    expect(screen.getByRole("slider")).toHaveAttribute("aria-valuenow", "0");
    expect(screen.getByRole("button", { name: "2GB" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );

    fireEvent.click(screen.getByRole("button", { name: "4GB" }));
    expect(onChange).toHaveBeenCalledWith(4096);

    fireEvent.keyDown(screen.getByRole("slider"), { key: "ArrowRight" });
    expect(onChange).toHaveBeenCalledWith(4096);
  });
});
