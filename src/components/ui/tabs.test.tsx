import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

describe("Tabs", () => {
  it("shows the selected panel", () => {
    render(
      <Tabs defaultValue="one">
        <TabsList>
          <TabsTrigger value="one">Um</TabsTrigger>
          <TabsTrigger value="two">Dois</TabsTrigger>
        </TabsList>
        <TabsContent value="one">Painel um</TabsContent>
        <TabsContent value="two">Painel dois</TabsContent>
      </Tabs>,
    );

    const snapshotTab = screen.getByRole("tab", { name: "Dois" });
    fireEvent.mouseDown(snapshotTab);
    fireEvent.click(snapshotTab);

    expect(screen.getByRole("tab", { name: "Dois" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(screen.getByText("Painel dois")).toBeVisible();
  });
});
