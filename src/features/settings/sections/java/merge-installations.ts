export function samePath(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

export function mergeInstallations(
  found: string[],
  runtimePath: string | null,
): string[] {
  const rows = [...found];
  if (runtimePath && !rows.some((path) => samePath(path, runtimePath))) {
    rows.unshift(runtimePath);
  }
  return rows;
}
