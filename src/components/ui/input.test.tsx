import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Input } from "@/components/ui/input";

describe("Input", () => {
  it("accepts typed text", () => {
    render(<Input aria-label="Nome" />);

    fireEvent.change(screen.getByRole("textbox", { name: "Nome" }), {
      target: { value: "Lilie" },
    });

    expect(screen.getByRole("textbox", { name: "Nome" })).toHaveValue("Lilie");
  });
});
