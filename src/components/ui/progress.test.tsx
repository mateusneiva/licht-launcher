import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Progress } from "@/components/ui/progress";

describe("Progress", () => {
  it("exposes the current value", () => {
    render(<Progress aria-label="Instalação" value={40} />);

    expect(
      screen.getByRole("progressbar", { name: "Instalação" }),
    ).toHaveAttribute("aria-valuenow", "40");
  });
});
