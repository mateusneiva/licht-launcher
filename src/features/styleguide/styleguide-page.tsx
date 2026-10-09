import { InfoIcon, MoonIcon, SunIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Input,
  Progress,
  ScrollArea,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Toaster,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui";

const colorTokens = [
  { name: "Background", className: "bg-background" },
  { name: "Text", className: "bg-foreground" },
  { name: "Primary", className: "bg-primary" },
  { name: "Secondary", className: "bg-secondary" },
  { name: "Muted", className: "bg-muted" },
  { name: "Destructive", className: "bg-destructive" },
  { name: "Card", className: "bg-card" },
  { name: "Border", className: "bg-border" },
] as const;

const radiusTokens = [
  { name: "Small", className: "rounded-sm" },
  { name: "Medium", className: "rounded-md" },
  { name: "Large", className: "rounded-lg" },
] as const;

const spacingTokens = [
  { name: "2", className: "p-2" },
  { name: "4", className: "p-4" },
  { name: "8", className: "p-8" },
] as const;

export function StyleguidePage() {
  const [isDark, setIsDark] = useState(() =>
    document.documentElement.classList.contains("dark"),
  );

  useEffect(() => {
    return () => {
      document.documentElement.classList.add("dark");
    };
  }, []);

  function toggleTheme() {
    const nextIsDark = !isDark;
    setIsDark(nextIsDark);
    document.documentElement.classList.toggle("dark", nextIsDark);
  }

  return (
    <TooltipProvider>
      <main className="mx-auto flex max-w-3xl flex-col gap-8 p-6">
        <header className="flex items-center justify-between gap-4">
          <div className="flex flex-col gap-1">
            <h1 className="text-xl font-medium">Style guide</h1>
            <p className="text-sm text-muted-foreground">
              Design tokens and base components for Licht Launcher.
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            aria-pressed={isDark}
            onClick={toggleTheme}
          >
            {isDark ? <SunIcon /> : <MoonIcon />}
            {isDark ? "Light theme" : "Dark theme"}
          </Button>
        </header>

        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-medium">Colors</h2>
          <ul className="grid grid-cols-4 gap-3">
            {colorTokens.map((token) => (
              <li key={token.name} className="flex flex-col gap-2">
                <div
                  className={`${token.className} size-10 rounded-md border border-border`}
                />
                <span className="text-sm">{token.name}</span>
              </li>
            ))}
          </ul>
        </section>

        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-medium">Typography</h2>
          <p className="text-sm">Small text</p>
          <p className="text-base">Body text</p>
          <p className="text-lg">Section title</p>
          <p className="flex items-center gap-2 text-xl">
            <InfoIcon />
            Page title
          </p>
        </section>

        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-medium">Radius and spacing</h2>
          <ul className="flex gap-4">
            {radiusTokens.map((token) => (
              <li key={token.name} className="flex flex-col items-center gap-2">
                <div
                  className={`${token.className} size-12 border border-border bg-muted`}
                />
                <span className="text-sm">{token.name}</span>
              </li>
            ))}
          </ul>
          <ul className="flex items-end gap-4">
            {spacingTokens.map((token) => (
              <li key={token.name}>
                <div className={`${token.className} bg-muted text-sm`}>
                  {token.name}
                </div>
              </li>
            ))}
          </ul>
        </section>

        <section className="flex flex-col gap-4">
          <h2 className="text-lg font-medium">Components</h2>
          <div className="flex flex-wrap gap-2">
            <Button type="button">Primary</Button>
            <Button type="button" variant="secondary">
              Secondary
            </Button>
            <Button type="button" variant="outline">
              Outline
            </Button>
            <Button type="button" variant="ghost">
              Ghost
            </Button>
            <Button type="button" variant="destructive">
              Destructive
            </Button>
          </div>

          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium" htmlFor="instance-name">
              Name
            </label>
            <Input id="instance-name" placeholder="Instance name" />
          </div>

          <Dialog>
            <DialogTrigger asChild>
              <Button type="button" variant="outline">
                Open dialog
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Dialog</DialogTitle>
                <DialogDescription>
                  Window used to confirm an action.
                </DialogDescription>
              </DialogHeader>
            </DialogContent>
          </Dialog>

          <Tabs defaultValue="release">
            <TabsList>
              <TabsTrigger value="release">Release</TabsTrigger>
              <TabsTrigger value="snapshot">Snapshot</TabsTrigger>
            </TabsList>
            <TabsContent value="release">Stable versions</TabsContent>
            <TabsContent value="snapshot">Test versions</TabsContent>
          </Tabs>

          <div className="flex flex-col gap-2">
            <span className="text-sm" id="install-progress-label">
              Progress
            </span>
            <Progress aria-labelledby="install-progress-label" value={60} />
          </div>

          <Select>
            <SelectTrigger aria-label="Channel">
              <SelectValue placeholder="Choose a channel" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="release">Release</SelectItem>
              <SelectItem value="snapshot">Snapshot</SelectItem>
            </SelectContent>
          </Select>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button type="button" variant="outline">
                Tip
              </Button>
            </TooltipTrigger>
            <TooltipContent>Help text</TooltipContent>
          </Tooltip>

          <ScrollArea className="h-24 rounded-md border border-border">
            <div className="flex flex-col gap-2 p-3 text-sm">
              <p>First line of the scrollable area.</p>
              <p>Second line of the scrollable area.</p>
              <p>Third line of the scrollable area.</p>
              <p>Fourth line of the scrollable area.</p>
            </div>
          </ScrollArea>

          <Button type="button" onClick={() => toast("Example notice")}>
            Show notice
          </Button>
        </section>
      </main>
      <Toaster theme={isDark ? "dark" : "light"} />
    </TooltipProvider>
  );
}
