import { useEffect } from "react";
import { toast } from "sonner";

import { friendlyError, messageOf } from "@/lib/errors";

export function useToastError(error: unknown) {
  useEffect(() => {
    if (error) {
      toast.error(friendlyError(messageOf(error)));
    }
  }, [error]);
}
