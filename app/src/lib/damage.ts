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

/** The damage Rules the run did not evaluate, or that do not apply, with the reason in the engine's words. */
export function damageNotes(trace: TraceLike | null | undefined): { rule: string; title: string; outcome: "not evaluated" | "not applicable"; sets: string[]; gap: string | null }[] {
  return (trace?.rows ?? [])
    .filter((r) => (isDamageRule(r.rule) || /damage/.test(r.rule ?? "")) && (r.outcome === "not evaluated" || r.outcome === "not applicable"))
    .map((r) => ({ rule: r.rule ?? "", title: r.title, outcome: r.outcome as "not evaluated" | "not applicable", sets: r.rule_sets, gap: r.gap }));
}
