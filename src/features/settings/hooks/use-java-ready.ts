import { useEffect, useState } from "react";

export function useJavaReady(visible: boolean) {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!visible) {
      setReady(false);
      return;
    }
    const handle = requestAnimationFrame(() => {
      setReady(true);
    });
    return () => {
      cancelAnimationFrame(handle);
    };
  }, [visible]);

  return ready;
}
