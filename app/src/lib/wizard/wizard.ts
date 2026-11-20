// The logic of the setup wizard (spec 0017, D2 and D3): which inputs each step asks, the limits, the Option
// name and the choices sent to the session call. No component code and no text: the components draw this
// and say it in the app's words. Choices, labels and limits come from the input schema, never from a list here.
import type { Choice, HardRange, InputSpec, Schema } from "../api";
import { RIVER_LIMITS } from "../inputsShown.ts";

/** The five steps and the review, in the order they are asked. */
export const STEPS = ["cargo", "area", "formation", "hull", "limits", "review"] as const;
export type StepId = (typeof STEPS)[number];

/** The optional inputs of step 5, in the order they are asked. The schema says which apply. */
export const OWN_LIMITS = ["moulded_depth_m", "max_speed_kn", "max_tank_od_m", "tanks_in_line", "min_freeboard_m"];
/** The Service Area whose Rules read the minimum freeboard; the river-only Rules do not. */
export const COASTAL = "coastal-and-river";
export const RIVER_ONLY = "river-only";

/** A number answer: `null` while empty, `NaN` while the text is not a number. */
export type Num = number | null;

export interface Answers {
  cargo: string | null;
  serviceArea: string | null;
  draughtLimit: Num;
  airDraftLimit: Num;
  formation: { kind: string; abreast: Num; inLine: Num } | null;
  /** How the Hull is defined: typed dimensions, or a standard Vessel Class. */
  hullMode: "typed" | "class";
  scheme: string | null;
  vesselClass: string | null;
  hullForm: string | null;
  length: Num;
  beam: Num;
  /** Step 5, by input name. An empty entry (or none) is left to the Defaults Profile. */
  own: Record<string, Num>;
  /** The designer's own Option name; `null` or blank means the made name. */
  optionName: string | null;
}

export function emptyAnswers(): Answers {
  return {
    cargo: null,
    serviceArea: null,
    draughtLimit: null,
    airDraftLimit: null,
    formation: null,
    hullMode: "typed",
    scheme: null,
    vesselClass: null,
    hullForm: null,
    length: null,
    beam: null,
    own: {},
    optionName: null,
  };
}

const spec = (schema: Schema, name: string): InputSpec | undefined => schema.inputs.find((i) => i.name === name);
const choices = (schema: Schema, name: string): Choice[] => spec(schema, name)?.choices ?? [];

/** The Cargo Containment a Cargo is carried in by default: its phase, read through the schema's own table. */
export function containmentOf(schema: Schema, cargo: string | null): string | null {
  if (cargo === null) return null;
  const phase = (choices(schema, "cargo").find((c) => c.value === cargo) as (Choice & { phase?: string }) | undefined)?.phase;
  const by = spec(schema, "cargo_containment")?.default_by;
  return (phase && by && (by.values[phase] as string | undefined)) || null;
}

/** Whether `s` is used for the Cargo and Formation chosen. Unknown choices: only what every choice uses. */
function appliesTo(schema: Schema, s: InputSpec, a: Answers): boolean {
  const covers = (list: string[] | undefined, all: Choice[], one: string | null) =>
    list === undefined || (one === null ? all.every((c) => list.includes(c.value)) : list.includes(one));
  const containment = containmentOf(schema, a.cargo);
  return (
    covers(s.applies_to, choices(schema, "cargo_containment"), containment) &&
    covers(s.applies_to_formation, choices(schema, "formation"), a.formation?.kind ?? null) &&
    covers((s as InputSpec & { applies_to_cargo?: string[] }).applies_to_cargo, choices(schema, "cargo"), a.cargo)
  );
}

export const riverOnly = (a: Answers) => a.serviceArea === RIVER_ONLY;

/** The names of the schema inputs a step asks, for the choices made so far. */
export function inputsOnStep(schema: Schema, a: Answers, step: StepId): string[] {
  switch (step) {
    case "cargo":
      return ["cargo"];
    case "area":
      return riverOnly(a) ? ["service_area", ...RIVER_LIMITS] : ["service_area"];
    case "formation":
      return ["formation"];
    case "hull":
      return a.hullMode === "class" ? ["hull_form", "vessel_scheme", "vessel_class"] : ["hull_form", "length_overall_m", "beam_m"];
    case "limits":
      return OWN_LIMITS.filter((name) => {
        const s = spec(schema, name);
        if (!s || !appliesTo(schema, s, a)) return false;
        return name !== "min_freeboard_m" || a.serviceArea === COASTAL;
      });
    case "review":
      return [];
  }
}

/** What a number does against the limits of its input: a hard limit blocks, a usual range only warns. */
export interface NumberCheck {
  block: boolean;
  /** The text is not a number. */
  nan: boolean;
  /** The hard limit the value breaks. */
  hard: HardRange | null;
  /** The usual range the value is outside. */
  soft: { min: number; max: number } | null;
}

export function checkNumber(s: Pick<InputSpec, "hard" | "soft">, n: number): NumberCheck {
  if (Number.isNaN(n)) return { block: true, nan: true, hard: null, soft: null };
  const h = s.hard;
  const broken = h !== undefined && ((h.gt !== undefined && !(n > h.gt)) || (h.ge !== undefined && !(n >= h.ge)) || (h.lt !== undefined && !(n < h.lt)) || (h.le !== undefined && !(n <= h.le)));
  const outside = s.soft !== undefined && (n < s.soft.min || n > s.soft.max);
  return { block: broken, nan: false, hard: broken ? (h ?? null) : null, soft: !broken && outside ? (s.soft ?? null) : null };
}

/** A thing to tell the designer on a step. `block` stops Next (and Create). */
export interface Problem {
  /** The step the fix is on. */
  step: StepId;
  input: string;
  kind: "required" | "number" | "hard" | "soft";
  block: boolean;
  check?: NumberCheck;
}

function numberProblem(schema: Schema, step: StepId, name: string, n: Num, required: boolean): Problem[] {
  const s = spec(schema, name);
  if (n === null) return required ? [{ step, input: name, kind: "required", block: true }] : [];
  if (!s) return [];
  const check = checkNumber(s, n);
  if (check.nan) return [{ step, input: name, kind: "number", block: true, check }];
  if (check.hard) return [{ step, input: name, kind: "hard", block: true, check }];
  if (check.soft) return [{ step, input: name, kind: "soft", block: false, check }];
  return [];
}

/** The Vessel Class chosen, if it fits the scheme and Formation chosen. */
export function classChosen(schema: Schema, a: Answers): (Choice & { draught_limit_m?: number | null; air_draft_limit_m?: number | null }) | null {
  if (a.hullMode !== "class" || a.vesselClass === null) return null;
  const c = choices(schema, "vessel_class").find((x) => x.value === a.vesselClass);
  if (!c || c.scheme !== a.scheme) return null;
  if (a.formation && c.formation !== undefined && c.formation !== a.formation.kind) return null;
  return c;
}

/** Whether the Vessel Class chosen gives both river limits. */
export function classGivesLimits(schema: Schema, a: Answers): boolean {
  const c = classChosen(schema, a);
  return c !== null && c.draught_limit_m != null && c.air_draft_limit_m != null;
}

/** What is wrong, or worth a warning, on a step. The review reports what still stops Create. */
export function problemsOnStep(schema: Schema, a: Answers, step: StepId): Problem[] {
  const out: Problem[] = [];
  const need = (input: string, present: boolean) => {
    if (!present) out.push({ step, input, kind: "required", block: true });
  };
  switch (step) {
    case "cargo":
      need("cargo", a.cargo !== null);
      break;
    case "area":
      need("service_area", a.serviceArea !== null);
      if (riverOnly(a)) {
        out.push(...numberProblem(schema, step, "draught_limit_m", a.draughtLimit, false));
        out.push(...numberProblem(schema, step, "air_draft_limit_m", a.airDraftLimit, false));
      }
      break;
    case "formation": {
      need("formation", a.formation !== null);
      if (a.formation?.kind === "pushed_convoy") {
        const convoy = spec(schema, "formation")?.convoy;
        for (const [input, n, limits] of [["abreast", a.formation.abreast, convoy?.abreast], ["in_line", a.formation.inLine, convoy?.in_line]] as const) {
          if (n === null) out.push({ step, input, kind: "required", block: true });
          else if (limits) {
            const check = checkNumber(limits, n);
            if (check.nan) out.push({ step, input, kind: "number", block: true, check });
            else if (check.hard) out.push({ step, input, kind: "hard", block: true, check });
            else if (check.soft) out.push({ step, input, kind: "soft", block: false, check });
          }
        }
      }
      break;
    }
    case "hull":
      need("hull_form", a.hullForm !== null);
      if (a.hullMode === "class") {
        need("vessel_scheme", a.scheme !== null);
        need("vessel_class", classChosen(schema, a) !== null);
      } else {
        out.push(...numberProblem(schema, step, "length_overall_m", a.length, true));
        out.push(...numberProblem(schema, step, "beam_m", a.beam, true));
      }
      break;
    case "limits":
      for (const name of inputsOnStep(schema, a, "limits")) out.push(...numberProblem(schema, step, name, a.own[name] ?? null, false));
      break;
    case "review":
      if (riverOnly(a) && !classGivesLimits(schema, a)) {
        // A river-only Option needs both numbers (issue #1034), unless a Vessel Class gives them.
        if (a.draughtLimit === null) out.push({ step: "area", input: "draught_limit_m", kind: "required", block: true });
        if (a.airDraftLimit === null) out.push({ step: "area", input: "air_draft_limit_m", kind: "required", block: true });
      }
      break;
  }
  return out;
}

/** Whether Create can be pressed: nothing on any step blocks. */
export function canCreate(schema: Schema, a: Answers): boolean {
  return STEPS.every((step) => !problemsOnStep(schema, a, step).some((p) => p.block));
}

const sizeText = (n: Num) => String(n);

/** The Option name made from the Cargo and the size, for example "LNG 140x29"; the designer's own wins. */
export function optionName(schema: Schema, a: Answers): string {
  if (a.optionName !== null && a.optionName.trim() !== "") return a.optionName.trim();
  const cargo = choices(schema, "cargo").find((c) => c.value === a.cargo)?.label;
  const size = a.hullMode === "class" ? a.vesselClass : a.length !== null && a.beam !== null ? `${sizeText(a.length)}x${sizeText(a.beam)}` : null;
  return [cargo, size].filter((x) => x).join(" ") || "Option 1";
}

/** The typed choices the session call takes: the answers that apply, and nothing else. */
export function choicesSent(schema: Schema, a: Answers): Record<string, unknown> {
  const sent: Record<string, unknown> = {};
  if (a.cargo !== null) sent.cargo = a.cargo;
  if (a.hullForm !== null) sent.hull_form = a.hullForm;
  if (a.formation !== null) {
    const f = a.formation;
    sent.formation = f.kind === "pushed_convoy" ? { kind: f.kind, abreast: f.abreast, in_line: f.inLine } : { kind: f.kind };
  }
  if (a.hullMode === "class") {
    if (a.scheme !== null) sent.vessel_scheme = a.scheme;
    if (a.vesselClass !== null) sent.vessel_class = a.vesselClass;
  } else {
    if (a.length !== null) sent.length_overall_m = a.length;
    if (a.beam !== null) sent.beam_m = a.beam;
  }
  if (a.serviceArea !== null) sent.service_area = a.serviceArea;
  if (riverOnly(a)) {
    if (a.draughtLimit !== null) sent.draught_limit_m = a.draughtLimit;
    if (a.airDraftLimit !== null) sent.air_draft_limit_m = a.airDraftLimit;
  }
  for (const name of inputsOnStep(schema, a, "limits")) {
    const n = a.own[name];
    if (n !== undefined && n !== null) sent[name] = n;
  }
  return sent;
}

/** How many inputs the Defaults Profile will fill: those that apply to this Design and are not sent. */
export function leftToDefaults(schema: Schema, a: Answers): number {
  const sent = choicesSent(schema, a);
  const known = (name: string) => sent[name] ?? spec(schema, name)?.default;
  return schema.inputs.filter((s) => {
    if (s.name in sent || s.kind === "vessel_scheme" || s.kind === "vessel_class" || RIVER_LIMITS.includes(s.name)) return false;
    if (!appliesTo(schema, s, a)) return false;
    const when = s.applies_when;
    const value = when ? known(when.input) : undefined;
    return !(when && value !== undefined && !when.values.includes(String(value)));
  }).length;
}

/** The wizard's answers for a set of typed inputs: an example, opened at the review step. */
export function answersFromInputs(inputs: Record<string, unknown>): Answers {
  const a = emptyAnswers();
  const num = (v: unknown): Num => (typeof v === "number" ? v : null);
  const text = (v: unknown): string | null => (typeof v === "string" ? v : null);
  a.cargo = text(inputs.cargo);
  a.serviceArea = text(inputs.service_area);
  a.draughtLimit = num(inputs.draught_limit_m);
  a.airDraftLimit = num(inputs.air_draft_limit_m);
  const f = inputs.formation as { kind?: string; abreast?: number; in_line?: number } | undefined;
  if (f && typeof f.kind === "string") a.formation = { kind: f.kind, abreast: num(f.abreast), inLine: num(f.in_line) };
  a.hullForm = text(inputs.hull_form);
  a.scheme = text(inputs.vessel_scheme);
  a.vesselClass = text(inputs.vessel_class);
  if (a.vesselClass !== null) a.hullMode = "class";
  a.length = num(inputs.length_overall_m);
  a.beam = num(inputs.beam_m);
  for (const name of OWN_LIMITS) if (num(inputs[name]) !== null) a.own[name] = num(inputs[name]);
  return a;
}
