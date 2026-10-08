import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("frontend startup", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("reports a missing application root", async () => {
    await expect(import("./main")).rejects.toThrow(
      "The application root element was not found.",
    );
  });

  it("mounts an empty frontend in the application root", async () => {
    render(<div id="root">Inicializando</div>);
    const root = screen.getByText("Inicializando");

    await act(async () => {
      await import("./main");
    });

    expect(root).toBeEmptyDOMElement();
  });
});
