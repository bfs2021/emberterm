// VS Code theme JSON → EmberTerm internal theme — Phase 1.
// Stub: will be fully implemented with AnsiPalette + UiColors conversion.

export interface VsCodeTheme {
  name: string;
  type: "dark" | "light";
  colors: Record<string, string>;
}

export function parseVscodeTheme(json: string): VsCodeTheme {
  return JSON.parse(json) as VsCodeTheme;
}
