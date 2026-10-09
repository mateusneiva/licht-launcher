import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("frontend startup", () => {
  beforeEach(() => {
    vi.resetModules();
    window.location.hash = "#/";
  });

  it("reports a missing application root", async () => {
    await expect(import("./main")).rejects.toThrow(
      "The application root element was not found.",
    );
  });

  it("keeps the style guide off the home route", async () => {
    render(<div id="root" />);

    await act(async () => {
      await import("./main");
    });

    expect(
      screen.queryByRole("heading", { name: "Style guide" }),
    ).not.toBeInTheDocument();
  });

  it("mounts the styleguide on the dev route", async () => {
    window.location.hash = "#/styleguide";
    render(<div id="root" />);

    await act(async () => {
      await import("./main");
    });

    expect(
      await screen.findByRole("heading", { name: "Style guide" }),
    ).toBeInTheDocument();
  });
});
