import { Link } from "@tanstack/react-router";

import { Button } from "@/components/ui/button";

export function AboutPage() {
  return (
    <main className="mx-auto flex w-full max-w-lg flex-col gap-4 p-6">
      <h1 className="text-xl font-semibold">Licht Launcher</h1>
      <p className="text-sm text-muted-foreground">Version 0.1.0, Lilie</p>
      <p className="text-sm">
        Unofficial project, not affiliated with Mojang Studios or Microsoft.
      </p>
      <Button variant="link" asChild className="self-start px-0">
        <Link to="/">Versions</Link>
      </Button>
    </main>
  );
}
