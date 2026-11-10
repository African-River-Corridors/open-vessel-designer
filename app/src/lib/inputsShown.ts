// Which inputs the left panel draws (issues #925, #1034). The Option holds an input, or the input is one
// the designer must answer before there are results; an unanswered one is drawn empty.
import type { InputSpec } from "./api";

/** The two river limits of a river-only Option: it holds no value for them until one is typed. */
export const RIVER_LIMITS = ["draught_limit_m", "air_draft_limit_m"];

/**
 * Whether the panel draws `spec`. `held` is the Option's inputs. A required choice the Option does not hold
 * (the Service Area, issue #925) is drawn with nothing chosen. A river-only Option has no river limit until
 * one is answered (issue #1034), so both limits are drawn empty and a number is required.
 */
export function isShown(spec: Pick<InputSpec, "name" | "kind" | "required">, held: Record<string, unknown>, riverOnly: boolean): boolean {
  if (held[spec.name] !== undefined) return true;
  if (spec.required && spec.kind === "choice") return true;
  return riverOnly && RIVER_LIMITS.includes(spec.name);
}
