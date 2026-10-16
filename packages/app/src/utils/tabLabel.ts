export interface LabelTab {
  id: string;
  title: string;
  customTitle: boolean;
  cwd?: string;
}

// Titles a shell sets on its own (prompt path, exe path, shell name). Anything else was set by a
// program the user is running (vim, claude, ...), which is more useful than the folder.
const SHELL_TITLE = [
  /^(MINGW(32|64)|MSYS|UCRT64|CLANG(32|64|ARM64)):/, // Git Bash
  /^[A-Za-z]:[\\/]/, // Windows path or exe path
  /^[/~]/, // Unix path
  /^[^@\s]+@[^:\s]+:/, // user@host: path (WSL / Linux)
  /^(cmd|pwsh|powershell|windows powershell|command prompt|bash|zsh|sh|wsl|git bash)(\.exe)?$/i,
];

function isShellTitle(title: string): boolean {
  return SHELL_TITLE.some((re) => re.test(title));
}

/** Last `depth` folders of a path, "/" for the Unix root. */
function folderName(cwd: string, depth: number): string {
  const segs = cwd.replace(/\\/g, "/").split("/").filter(Boolean);
  if (segs.length === 0) return "/";
  return segs.slice(-depth).join("/");
}

/**
 * Tab text: the user's own name, else a program's title, else the last folder of the shell's
 * working directory. Tabs that would read the same get their parent folder added.
 */
export function tabLabels(tabs: LabelTab[]): Map<string, string> {
  const labels = new Map<string, string>();
  const byFolder: LabelTab[] = [];
  for (const t of tabs) {
    if (!t.customTitle && t.cwd && isShellTitle(t.title)) byFolder.push(t);
    else labels.set(t.id, t.title);
  }

  let pending = byFolder;
  for (let depth = 1; pending.length > 0; depth++) {
    const names = pending.map((t) => folderName(t.cwd!, depth));
    const counts = new Map<string, number>();
    names.forEach((n) => counts.set(n, (counts.get(n) ?? 0) + 1));
    const stillAmbiguous: LabelTab[] = [];
    pending.forEach((t, i) => {
      // Give up widening after 4 levels, or when the path has nothing more to add.
      const exhausted = depth >= 4 || folderName(t.cwd!, depth + 1) === names[i];
      if ((counts.get(names[i]) ?? 0) > 1 && !exhausted) stillAmbiguous.push(t);
      else labels.set(t.id, names[i]);
    });
    pending = stillAmbiguous;
  }
  return labels;
}
