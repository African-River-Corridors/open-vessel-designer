// The damage cases of an Option, read from its results for the Damage stability view. The app computes
// nothing here: it finds the case lists, groups them by loading condition and finds each case's row in
// the Rule Trace. The governing case of a condition is the first case whose damage Rule fails in the
// trace; where every case passes, it is the case the engine names as governing (the `governing` list of
// the `damage` block and of each `damage_by_rule` set), which has a row in the trace too.

export interface DamageStage {
  fraction: number;
  floats: string;
  heel_deg: number;
  max_gz_m: number;
  area_m_rad: number;
  ok: boolean;
}
export interface DamageCase {
  name: string;
  /** The loading condition, where the Rule Set gives the cases per condition. */
  condition?: string;
  flooded: string[];
  floats: string;
  heel_deg: number;
  trim_deg: number;
  draught_mid_m: number;
  max_gz_m: number;
  range_end_deg: number;
  area_m_rad: number;
  ok: boolean;
  stages?: DamageStage[];
}
interface TraceRow {
  rule: string | null;
  rule_sets: string[];
  title: string;
  object: string | null;
  outcome: string;
  gap: string | null;
}
export interface TraceLike {
  rows: TraceRow[];
}
/** The results blocks this module reads. */
export interface DamageResults {
  damage?: unknown;
  marpol_damage?: DamageCase[];
  damage_by_rule?: Record<string, { rule_set?: string; method?: string; cases?: DamageCase[]; governing?: { condition: string; case: string }[] }>;
  rule_trace?: TraceLike;
  /** Issue #935: whether the arrangement was screened by damage; null where there is no IGC damage to screen by. */
  damage_screening?: { screened: boolean; note?: string | null; changed_note?: string | null; limit_note?: string | null; planned_runs?: number | null } | null;
}

/** One list of damage cases and what the engine says about it. */
export interface DamageSet {
  /** A key for the list, stable between runs. */
  id: string;
  /** The Rule Set and version (`igc-2g@3.1.0`), where the results name one. */
  ruleSet: string | null;
  /** The method, in the engine's words, where the results give one. */
  method: string | null;
  cases: DamageCase[];
  /** The governing case the engine names, by loading condition (the gas path: the `damage` block and each `damage_by_rule` set). */
  engineGoverning: { condition: string; case: string }[];
  /** The trace rows of the cases are the rows of this Rule, by its id prefix (and Rule Set, where known). */
  rulePrefix: string;
}

/** Whether a trace Rule id is a damage survival or damage stability Rule. */
export const isDamageRule = (rule: string | null) => !!rule && /\/(damage-(survival|stability)|standard-damage-cases)$/.test(rule);

const RULE_PREFIX: Record<string, string> = { "igc-2g": "igc/", "es-trin-2025": "es-trin/", "normam-202": "normam/", "adn-gas": "adn/" };
const setId = (key: string) => key.split("@")[0];

/** The case lists of the results: the `damage` block (an object of the gas path, a list of the liquid path), then `damage_by_rule`, then MARPOL. */
export function damageSets(results: DamageResults | null | undefined): DamageSet[] {
  if (!results) return [];
  const out: DamageSet[] = [];
  const d = results.damage;
  if (Array.isArray(d) && d.length > 0) {
    out.push({ id: "damage", ruleSet: null, method: null, cases: d as DamageCase[], engineGoverning: [], rulePrefix: "adn/" });
  } else if (d && typeof d === "object" && Array.isArray((d as { cases?: unknown }).cases)) {
    const b = d as { rule_set?: string; method?: string; cases: DamageCase[]; governing?: { condition: string; case: string }[] };
    if (b.cases.length > 0) {
      out.push({
        id: "damage",
        ruleSet: b.rule_set ?? null,
        method: b.method ?? null,
        cases: b.cases,
        engineGoverning: (b.governing ?? []).map((g) => ({ condition: g.condition, case: g.case })),
        rulePrefix: RULE_PREFIX[setId(b.rule_set ?? "")] ?? "",
      });
    }
  }
  for (const [key, b] of Object.entries(results.damage_by_rule ?? {})) {
    if (b?.cases?.length) {
      out.push({
        id: `by-rule:${key}`,
        ruleSet: b.rule_set ?? key,
        method: b.method ?? null,
        cases: b.cases,
        engineGoverning: (b.governing ?? []).map((g) => ({ condition: g.condition, case: g.case })),
        rulePrefix: RULE_PREFIX[setId(b.rule_set ?? key)] ?? "",
      });
    }
  }
  if (results.marpol_damage?.length) {
    out.push({ id: "marpol", ruleSet: null, method: null, cases: results.marpol_damage, engineGoverning: [], rulePrefix: "marpol/" });
  }
  return out;
}

/** The object name the trace uses for a case. */
export const objectOf = (c: DamageCase) => (c.condition ? `${c.condition}: ${c.name}` : c.name);

/**
 * The trace row of a case, or undefined. Two cases can have one name (a bottom damage at two depths); the
 * trace then holds one row for each, in the order of the cases, so the nth case of a name takes the nth row.
 */
export function rowOf(set: DamageSet, c: DamageCase, trace: TraceLike | null | undefined): TraceRow | undefined {
  const o = objectOf(c);
  const nth = set.cases.filter((k) => objectOf(k) === o).indexOf(c);
  const rows = (trace?.rows ?? []).filter(
    (r) => r.object === o && isDamageRule(r.rule) && (r.rule ?? "").startsWith(set.rulePrefix) && (!set.ruleSet || r.rule_sets.includes(set.ruleSet)),
  );
  return rows[Math.max(nth, 0)];
}

/** A case's verdict in the trace; the case's own `ok` where the trace has no row for it. */
export const passes = (set: DamageSet, c: DamageCase, trace: TraceLike | null | undefined): boolean => {
  const row = rowOf(set, c, trace);
  return row ? row.outcome !== "fail" : c.ok;
};

/** The loading conditions of a list, in the order the cases give them (one empty condition where the cases have none). */
export function conditionsOf(set: DamageSet): string[] {
  const seen: string[] = [];
  for (const c of set.cases) if (!seen.includes(c.condition ?? "")) seen.push(c.condition ?? "");
  return seen;
}

/**
 * The governing case of a loading condition, as the engine names it (the worst case: a failing one beats a
 * passing one, then the least maximum righting lever). A list with no engine governing case (the liquid
 * path) falls back to its first case that fails in the trace, and null where none fails.
 */
export function governingCase(set: DamageSet, condition: string, trace: TraceLike | null | undefined): DamageCase | null {
  const mine = set.cases.filter((c) => (c.condition ?? "") === condition);
  if (set.engineGoverning.length > 0) {
    const named = set.engineGoverning.find((g) => g.condition === condition);
    return (named && mine.find((c) => c.name === named.case)) || null;
  }
  return mine.find((c) => !passes(set, c, trace)) ?? null;
}

/** The damage Rules the run did not evaluate, did not run (issue #923), or that do not apply, with the reason in the engine's words. */
export function damageNotes(trace: TraceLike | null | undefined): { rule: string; title: string; outcome: "not evaluated" | "not run" | "not applicable"; sets: string[]; gap: string | null }[] {
  return (trace?.rows ?? [])
    .filter((r) => (isDamageRule(r.rule) || /damage/.test(r.rule ?? "")) && (r.outcome === "not evaluated" || r.outcome === "not run" || r.outcome === "not applicable"))
    .map((r) => ({ rule: r.rule ?? "", title: r.title, outcome: r.outcome as "not evaluated" | "not run" | "not applicable", sets: r.rule_sets, gap: r.gap }));
}

// ---- Damage on demand (issue #924) ----
// The engine runs damage only when asked. The time estimate is measured here, in the front end (the engine
// reads no clock): time per case = elapsed / cases done, and the time left = cases left x time per case.

/** Whether damage stability was run for the shown results: the engine's `damage_status` (issue #923), with its case count. */
export interface DamageStatus {
  status: "run" | "not run" | "not applicable" | "not evaluated" | null;
  /** The cases the run assesses (counted even when damage is skipped); null where the results give none. */
  cases: number | null;
}

/** The status of the damage run in the results; `status` is null for results from an engine that gives none. */
export function damageStatus(results: { damage_status?: unknown; damage_cases?: unknown } | null | undefined): DamageStatus {
  const s = results?.damage_status;
  const known = s === "run" || s === "not run" || s === "not applicable" || s === "not evaluated";
  const n = results?.damage_cases;
  return { status: known ? s : null, cases: typeof n === "number" && Number.isFinite(n) ? n : null };
}

/**
 * The cases that must be done before the time left is shown: a few, or 5 % of the total. The first cases of a
 * run are cheaper than the rest (measured in the e2e run: the first case took a quarter of the average), so
 * a time per case from one or two of them is too short by a factor of 3 or more.
 */
export const minSample = (total: number): number => Math.max(3, Math.ceil(total * 0.05));

/**
 * Seconds left in a run: the cases left times the time per case so far (elapsed over done, with `elapsedMs`
 * counted from the first progress event, so the set-up before the cases is not in it). null until `minSample`
 * cases are done, or where the total or the time is not usable. 0 when every case is done.
 */
export function secondsLeft(done: number, total: number, elapsedMs: number): number | null {
  if (!(total > 0) || !(done >= 0) || !(elapsedMs >= 0)) return null;
  if (done >= total) return 0;
  if (done < minSample(total) || elapsedMs <= 0) return null;
  return ((total - done) * (elapsedMs / done)) / 1000;
}

/** The time a whole run is expected to take, from a time per case measured on this machine; null with no measure or no cases. */
export function secondsBefore(cases: number | null, secondsPerCase: number | null): number | null {
  if (cases === null || secondsPerCase === null || !(cases > 0) || !(secondsPerCase > 0)) return null;
  return cases * secondsPerCase;
}

/** The time per case of a finished run (seconds); null where it cannot be told. */
export function secondsPerCase(total: number, elapsedMs: number): number | null {
  return total > 0 && elapsedMs > 0 ? elapsedMs / total / 1000 : null;
}

/** Seconds as the app shows them: whole seconds, never less than 1. */
export const wholeSeconds = (s: number): number => Math.max(1, Math.round(s));

const PER_CASE_KEY = "ovd.damageSecondsPerCase";

/** The time per case kept from the last run on this machine, or null. Storage may be missing or throw. */
export function loadSecondsPerCase(): number | null {
  try {
    const v = Number(window.localStorage.getItem(PER_CASE_KEY));
    return Number.isFinite(v) && v > 0 ? v : null;
  } catch {
    return null;
  }
}
export function saveSecondsPerCase(v: number): void {
  try {
    window.localStorage.setItem(PER_CASE_KEY, String(v));
  } catch {
    /* not kept: the next run shows the case count only */
  }
}

/**
 * The Message Ids of the question before a report with damage not run (design of issue #924, section 7). The
 * app has no report yet; the ticket that adds one uses these words as they are.
 */
export const REPORT_DAMAGE_PROMPT = {
  title: "app-report-damage-title",
  bodyTime: "app-report-damage-body-time",
  bodyCases: "app-report-damage-body-cases",
  run: "app-report-damage-run",
  skip: "app-report-damage-skip",
  cancel: "app-report-damage-cancel",
} as const;
