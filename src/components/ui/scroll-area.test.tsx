import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ScrollArea } from "@/components/ui/scroll-area";

describe("ScrollArea", () => {
  it("keeps content inside the scrollable viewport", () => {
    render(
      <ScrollArea>
        <button type="button">Rolar</button>
      </ScrollArea>,
    );

    const button = screen.getByRole("button", { name: "Rolar" });

    button.focus();

    expect(button.closest("[data-slot=scroll-area-viewport]")).not.toBeNull();
    expect(button).toHaveFocus();
  });
});
