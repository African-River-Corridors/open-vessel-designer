// The outcomes of a Rule row, and how the app shows each one (issue #923 adds `not run`).
// `not run`: the engine can evaluate the Rule and the run left it out on purpose (damage stability with damage
// skipped). It is not `not evaluated` (an engine gap) and it is never a pass.

export type Outcome = "pass" | "fail" | "warn" | "note" | "not evaluated" | "not run" | "not applicable";

/** The display order of the outcomes, and each one's tag: a word and a shape, never colour only. */
export const OUTCOMES: Record<Outcome, { rank: number; mark: string; label: string; tone: string }> = {
  fail: { rank: 0, mark: "✕", label: "app-fail", tone: "fail" },
  warn: { rank: 1, mark: "!", label: "app-rules-warn", tone: "warn" },
  pass: { rank: 2, mark: "✓", label: "app-pass", tone: "ok" },
  note: { rank: 3, mark: "i", label: "app-rules-note", tone: "plain" },
  "not evaluated": { rank: 4, mark: "○", label: "app-rules-not-evaluated", tone: "open" },
  "not run": { rank: 5, mark: "◌", label: "app-rules-not-run", tone: "open" },
  "not applicable": { rank: 6, mark: "–", label: "app-rules-not-applicable", tone: "plain" },
};

/** The rank of an outcome; an outcome this build does not know sorts last and never throws. */
export const outcomeRank = (outcome: string): number => (OUTCOMES as Record<string, { rank: number }>)[outcome]?.rank ?? 99;

/** The tag of an outcome; an unknown outcome gets a plain tag of its own words. */
export function outcomeTag(outcome: string): { label: string | null; tone: string } {
  const o = (OUTCOMES as Record<string, { label: string; tone: string }>)[outcome];
  return o ? { label: o.label, tone: o.tone } : { label: null, tone: "plain" };
}
