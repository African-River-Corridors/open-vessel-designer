// The Summary view's lines (issue #1081, spec 0017 D7). It turns an evaluation of one Option into the
// verdict, the headline numbers and the things to look at next. It calculates nothing: every number and text
// is read from the results the engine gave, and the component formats each number as the Outputs panel does
// (metrics.ts). No component code and no i18n here, so `npm run test:unit` runs it under node.
import { damageStatus, type DamageStatus } from "./damage.ts";

/** An input of the Option that has a hard error, with the label the Input panel shows. */
export interface Blocker {
  name: string;
  label: string;
}

export type Verdict =
  /** Every evaluated blocking Rule is met. */
  | { kind: "met" }
  /** Blocking Rules that are not met, by title; checks of severity error that fail are named too. */
  | { kind: "not-met"; titles: string[] }
  /** No results: the inputs to fix, or the engine's own message where the Option has no results for another reason. */
  | { kind: "not-calculated"; inputs: Blocker[]; message: string | null };

/** One headline number: where it lives in the results, and its value there. The component formats it. */
export interface SummaryRow {
  /** The Message Id of the label; the same one the Outputs panel uses for the same number. */
  label: string;
  path: string[];
  value: unknown;
}

export interface SummaryWarning {
  name: string;
  message: string;
}

export interface Summary {
  verdict: Verdict;
  /** The scope sentence of the Rule Trace (which Rule Sets were checked, how many blocking Rules were not evaluated). */
  scope: string | null;
  damage: DamageStatus;
  rows: SummaryRow[];
  /** The failed checks of severity warning, in the Checks panel's order. */
  warnings: SummaryWarning[];
  inputs: { typed: number; defaults: number };
}

/** What the function reads of an Option: its outcome and the inputs as stored. */
export interface SummaryInput {
  /** The Option's results object, or null where the Option has none (not calculated). */
  results: Record<string, unknown> | null;
  /** The engine's message where the Option has no results and no input to fix (not available, failed). */
  message: string | null;
  blockers: Blocker[];
  /** The stored inputs of the Option, each with its source. */
  inputs: Record<string, { source: string }>;
}

const at = (v: unknown, path: string[]): unknown => {
  for (const key of path) {
    if (v === null || typeof v !== "object") return undefined;
    // A key `name=X` picks the list item named X, as in metrics.ts.
    v = Array.isArray(v)
      ? v.find((x) => x !== null && typeof x === "object" && key.startsWith("name=") && (x as { name?: unknown }).name === key.slice(5))
      : (v as Record<string, unknown>)[key];
  }
  return v;
};
const block = (v: unknown, path: string[]): boolean => {
  const b = at(v, path);
  return b !== null && b !== undefined && typeof b === "object";
};
const row = (label: string, ...path: string[]): Omit<SummaryRow, "value"> => ({ label, path });

// The labels are the Message Ids of `MAIN`, `TANKS` and `POWERING` in metrics.ts, so a number reads as in Outputs.
const CARGO = [
  row("app-metric-cargo-volume", "results", "cargo_volume_m3"),
  row("app-metric-cargo", "results", "cargo_t"),
  row("app-metric-governs", "results", "governs"),
];
const TANKS = [
  row("app-metric-tanks-across", "arrangement", "n_wide"),
  row("app-metric-tanks-along", "arrangement", "n_long"),
  row("app-metric-tank-od", "arrangement", "tank_od_m"),
];
const DRAUGHT = [
  row("app-metric-draught", "results", "draught_m"),
  row("app-metric-draught-max", "results", "draught_max_m"),
  row("app-metric-freeboard", "results", "freeboard_m"),
  row("app-metric-lightship", "results", "lightship_t"),
  row("app-metric-displacement", "results", "displacement_t"),
];
const SPEEDS = [
  row("app-metric-service-speed-deep", "powering", "environments", "name=Deep water", "service_speed_kn"),
  row("app-metric-service-speed-channel", "powering", "environments", "name=Channel", "service_speed_kn"),
];
const AIR = [row("app-metric-air-draft-loaded", "results", "air_draft_loaded_m"), row("app-metric-air-draft-light", "results", "air_draft_light_m")];

/** The headline numbers of a results object: the tank lines only for a Cargo in tanks, the power lines only where the Option has a Propulsor. */
export function summaryRows(results: unknown): SummaryRow[] {
  const power = block(results, ["powering"]);
  return [
    ...CARGO,
    ...(block(results, ["arrangement"]) ? TANKS : []),
    ...DRAUGHT,
    ...(power ? [row("app-metric-installed-power", "powering", "installed_power_kw")] : []),
    // A service speed line only for an environment the powering block has.
    ...(power ? SPEEDS.filter((r) => block(results, r.path.slice(0, -1))) : []),
    ...AIR,
  ].map((r) => ({ ...r, value: at(results, r.path) }));
}

interface TraceRow {
  title: string;
  consequence: string | null;
  outcome: string;
}
interface Check {
  name: string;
  ok: boolean;
  message: string;
  severity?: string;
}

export function summarise(i: SummaryInput): Summary {
  const results = i.results;
  const sources = Object.values(i.inputs).map((v) => v.source);
  const typed = sources.filter((s) => s === "typed").length;
  const inputs = { typed, defaults: sources.length - typed };
  if (!results) {
    return {
      verdict: { kind: "not-calculated", inputs: i.blockers, message: i.message },
      scope: null,
      damage: damageStatus(null),
      rows: [],
      warnings: [],
      inputs,
    };
  }
  const trace = at(results, ["rule_trace"]) as { rows?: TraceRow[]; scope?: string } | undefined;
  const checks = (at(results, ["checks"]) as Check[] | undefined) ?? [];
  // A blocking Rule that fails, by title; a check of severity error that fails (a check with no severity is an error, as in the Checks panel).
  const titles: string[] = [];
  for (const r of trace?.rows ?? []) if (r.consequence === "block" && r.outcome === "fail" && !titles.includes(r.title)) titles.push(r.title);
  for (const c of checks) if (!c.ok && c.severity !== "warning" && !titles.includes(c.name)) titles.push(c.name);
  return {
    verdict: titles.length === 0 ? { kind: "met" } : { kind: "not-met", titles },
    scope: typeof trace?.scope === "string" ? trace.scope : null,
    damage: damageStatus(results),
    rows: summaryRows(results),
    warnings: checks.filter((c) => !c.ok && c.severity === "warning").map((c) => ({ name: c.name, message: c.message })),
    inputs,
  };
}
