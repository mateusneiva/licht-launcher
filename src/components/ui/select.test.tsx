import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

describe("Select", () => {
  it("chooses an option", async () => {
    render(
      <Select>
        <SelectTrigger aria-label="Canal">
          <SelectValue placeholder="Escolha" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="release">Release</SelectItem>
        </SelectContent>
      </Select>,
    );

    fireEvent.click(screen.getByRole("combobox", { name: "Canal" }));
    fireEvent.click(await screen.findByRole("option", { name: "Release" }));

    expect(screen.getByRole("combobox", { name: "Canal" })).toHaveTextContent(
      "Release",
    );
  });
});
