import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { StyleguidePage } from "@/features/styleguide/styleguide-page";

describe("StyleguidePage", () => {
  it("toggles the dark theme class", () => {
    render(<StyleguidePage />);

    expect(
      screen.getByRole("heading", { name: "Style guide" }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Dark theme" }));
    expect(document.documentElement).toHaveClass("dark");

    fireEvent.click(screen.getByRole("button", { name: "Light theme" }));
    expect(document.documentElement).not.toHaveClass("dark");
  });
});
