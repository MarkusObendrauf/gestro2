export type Direction = "N" | "NE" | "E" | "SE" | "S" | "SW" | "W" | "NW";

export const ALL_DIRECTIONS: Direction[] = [
  "N",
  "NE",
  "E",
  "SE",
  "S",
  "SW",
  "W",
  "NW",
];

export interface Shortcut {
  modifiers: string[];
  key: string;
  label?: string;
}

export interface GestroConfig {
  threshold: number;
  bindings: Record<string, Shortcut>;
  launch_at_login: boolean;
}

export const DIRECTION_LABELS: Record<Direction, string> = {
  N: "Up",
  NE: "Up-Right",
  E: "Right",
  SE: "Down-Right",
  S: "Down",
  SW: "Down-Left",
  W: "Left",
  NW: "Up-Left",
};

/** Format a shortcut for display. */
export function formatShortcut(shortcut: Shortcut): string {
  const parts = [...shortcut.modifiers, shortcut.key];
  return parts.join(" + ");
}
