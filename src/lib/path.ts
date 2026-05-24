/** Path relative to `root`, or `full` when not under root (case-insensitive on Windows). */
export function relativeToRoot(full: string, root: string): string {
  const normRoot = root.replace(/\\/g, "/").replace(/\/$/, "");
  const norm = full.replace(/\\/g, "/");
  if (norm.toLowerCase().startsWith(normRoot.toLowerCase())) {
    return norm.slice(normRoot.length).replace(/^\//, "") || full;
  }
  return full;
}
