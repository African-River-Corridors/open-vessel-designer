// The principal characteristics strip (issue #889). The rows, their order, units and decimals come from one data
// file, `catalogue/report/principal_characteristics.json`, which the report summary (tools/concept_report.py,
// spec 0015 section 4.1) reads too. This module only reads values from the run's results; it computes nothing.
import file from "../../../catalogue/report/principal_characteristics.json" with { type: "json" };

export interface PrincipalRow {
  id: string;
  /** Key path in the Option's results; a key `name=X` picks the list item named X. */
  path: string[];
  /** The Message Id of the label. */
  label_id: string;
  /** The English label (the report uses it; the app shows the Message Id's text). */
  label: string;
  unit: string;
  decimals: number;
}
export const PRINCIPAL_ROWS: PrincipalRow[] = file.rows;

export interface PrincipalItem {
  id: string;
  labelId: string;
  unit: string;
  /** The value as text, or `notEvaluated` when the run does not hold a number there. */
  text: string;
  evaluated: boolean;
}

function pick(results: unknown, path: string[]): unknown {
  let v: unknown = results;
  for (const key of path) {
    if (v === null || typeof v !== "object") return undefined;
    if (Array.isArray(v) && key.startsWith("name=")) {
      v = v.find((x) => x !== null && typeof x === "object" && (x as { name?: unknown }).name === key.slice(5));
    } else {
      v = (v as Record<string, unknown>)[key];
    }
  }
  return v;
}

/** The strip of one Option: every row of the data file, in its order. `results` null (no run) leaves every value not evaluated. */
export function principalItems(
  results: unknown,
  format: (value: number, decimals: number) => string,
  notEvaluated: string,
): PrincipalItem[] {
  return PRINCIPAL_ROWS.map((r) => {
    const v = pick(results, r.path);
    const ok = typeof v === "number" && Number.isFinite(v);
    return { id: r.id, labelId: r.label_id, unit: r.unit, text: ok ? format(v, r.decimals) : notEvaluated, evaluated: ok };
  });
}
