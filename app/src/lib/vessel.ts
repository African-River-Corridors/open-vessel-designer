// The Summary's picture of the vessel: the side elevation and the plan the Drawings view shows, framed on the
// hull. Nothing is drawn here: the component shows the same sheet with only its hull geometry, and this module
// picks the frame from the boxes the page measures, and the dimensions the picture's text names. No component
// code, so `npm run test:unit` runs it under node.

/** A box in the sheet's own units (millimetres of the A3 sheet). */
export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The selector of the hull geometry in a sheet: the main viewport, without its dimensions, leaders and scale bar. */
export const HULL_GEOMETRY = '[data-origin="viewport.main"]';

/**
 * The frame of the hull: the union of its boxes, with `margin` (a fraction of the longer side) on every side.
 * A line has no width or no height and still counts; a box with neither counts not. Null when nothing counts.
 */
export function frameOf(boxes: Box[], margin: number): Box | null {
  const kept = boxes.filter((b) => b.width > 0 || b.height > 0);
  if (kept.length === 0) return null;
  const left = Math.min(...kept.map((b) => b.x));
  const top = Math.min(...kept.map((b) => b.y));
  const right = Math.max(...kept.map((b) => b.x + b.width));
  const bottom = Math.max(...kept.map((b) => b.y + b.height));
  const pad = margin * Math.max(right - left, bottom - top);
  return { x: left - pad, y: top - pad, width: right - left + 2 * pad, height: bottom - top + 2 * pad };
}

/** A length in metres as the picture's text says it: at most two decimals, no trailing zeros. */
const metres = (v: number) => String(Number(v.toFixed(2)));

/** The Option's length overall, beam and moulded depth, from the results; null unless all three are numbers. */
export function vesselDims(results: unknown): { loa: string; beam: string; depth: string } | null {
  const dims = (results as { dims?: Record<string, unknown> } | null)?.dims;
  const [loa, beam, depth] = ["loa_m", "beam_m", "depth_m"].map((k) => dims?.[k]);
  const ok = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);
  if (!ok(loa) || !ok(beam) || !ok(depth)) return null;
  return { loa: metres(loa), beam: metres(beam), depth: metres(depth) };
}
