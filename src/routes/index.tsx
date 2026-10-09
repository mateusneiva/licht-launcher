import { createFileRoute } from "@tanstack/react-router";

import { VersionsPage } from "@/features/versions/versions-page";

export const Route = createFileRoute("/")({
  component: VersionsPage,
});
