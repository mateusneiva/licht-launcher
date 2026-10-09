import { createFileRoute, notFound } from "@tanstack/react-router";

import { StyleguidePage } from "@/features/styleguide/styleguide-page";

export const Route = createFileRoute("/styleguide")({
  beforeLoad: () => {
    if (import.meta.env.VITE_SHOW_STYLEGUIDE !== "true") {
      throw notFound();
    }
  },
  component: StyleguidePage,
});
