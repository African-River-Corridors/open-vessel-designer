// Which inputs the Input panel draws in Essentials, and which the filter box finds (issue #1084, spec 0017
// D9 and D10). Pure functions: no component code, so unit tests can run them.
import type { InputSpec, StoredInput } from "./api";
import { isShown, RIVER_LIMITS } from "./inputsShown.ts";

/** Essentials draws the inputs a first Design needs; All inputs draws every input the Option holds. */
export type InputsMode = "essentials" | "all";

export const MODE_KEY = "ovd.inputsMode";
export const DEFAULT_MODE: InputsMode = "essentials";
const MODES: readonly InputsMode[] = ["essentials", "all"];

type Held = Record<string, Pick<StoredInput, "source"> | undefined>;
type Spec = Pick<InputSpec, "name" | "kind" | "required" | "essential">;
type Store = Pick<Storage, "getItem" | "setItem">;

/** The saved choice, or Essentials when none is kept or storage is not available. */
export function loadMode(storage: Pick<Storage, "getItem"> = localStorage): InputsMode {
  try {
    const saved = storage.getItem(MODE_KEY);
    return MODES.find((m) => m === saved) ?? DEFAULT_MODE;
  } catch {
    return DEFAULT_MODE;
  }
}

export function saveMode(mode: InputsMode, storage: Store = localStorage) {
  try {
    storage.setItem(MODE_KEY, mode);
  } catch {
    // Not remembered; the choice still applies for this run.
  }
}

/**
 * Whether Essentials draws `spec`: the Option must hold it (see isShown), and it is required, marked
 * essential in the schema, or typed by the designer (source typed or class).
 */
export function essentialsShown(spec: Spec, held: Held, riverOnly: boolean): boolean {
  if (!isShown(spec, held, riverOnly)) return false;
  // A river-only Option must be given both river limits (issue #1034), so they are required here.
  if (spec.required || spec.essential || (riverOnly && RIVER_LIMITS.includes(spec.name))) return true;
  const source = held[spec.name]?.source;
  return source === "typed" || source === "class";
}

/** Whether `label` holds `query`, without regard to case. Blank text matches every label. */
export function filterMatches(label: string, query: string): boolean {
  const q = query.trim().toLocaleLowerCase();
  return q === "" || label.toLocaleLowerCase().includes(q);
}

/**
 * The inputs to draw. With text in the filter box: every held input whose label (in the Display Language,
 * from `labelOf`) holds the text, in either mode. Without: all held inputs, or the Essentials ones.
 */
export function shownInputs<T extends Spec>(
  specs: T[],
  held: Held,
  riverOnly: boolean,
  mode: InputsMode,
  query: string,
  labelOf: (spec: T) => string,
): T[] {
  if (query.trim() !== "") return specs.filter((s) => isShown(s, held, riverOnly) && filterMatches(labelOf(s), query));
  return specs.filter((s) => (mode === "all" ? isShown(s, held, riverOnly) : essentialsShown(s, held, riverOnly)));
}

/** How many inputs All inputs draws that Essentials does not: they all use the Defaults Profile. */
export function hiddenByEssentials(specs: Spec[], held: Held, riverOnly: boolean): number {
  return specs.filter((s) => isShown(s, held, riverOnly) && !essentialsShown(s, held, riverOnly)).length;
}
