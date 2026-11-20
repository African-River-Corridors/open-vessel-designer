// The platform, made once at start: the Web adapter over the WebAssembly session. The desktop app is
// paused (ADR 0026); a returning desktop adapter implements the same `Platform` interface.
import type { Platform } from "./types";

export type {
  Platform,
  DrawingFormat,
  DrawingView,
  ModelFormat,
  Meshes,
  MeshGroup,
  SectionOutline,
  SpaceInfo,
  DeckItem,
  DeckItems,
  PropulsorPart,
  Waterline,
  DamageProgress,
  DamageEnd,
} from "./types";

let current: Platform | null = null;

export async function initPlatform(): Promise<Platform> {
  if (current) return current;
  current = await (await import("./web")).createWebPlatform();
  return current;
}

/** The chosen platform. Call `initPlatform()` first (main.ts does). */
export function platform(): Platform {
  if (!current) throw new Error("the platform is not ready");
  return current;
}
