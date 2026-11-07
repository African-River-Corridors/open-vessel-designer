// The figures of the Stern & crew view (issue #887), read from the results as the engine gives them.
// Nothing here computes a result: each figure is one key of the run, or null when the run has none.

type Obj = Record<string, unknown>;
const obj = (v: unknown): Obj | null => (v && typeof v === "object" && !Array.isArray(v) ? (v as Obj) : null);
const numOf = (v: unknown): number | null => (typeof v === "number" && Number.isFinite(v) ? v : null);
const strOf = (v: unknown): string | null => (typeof v === "string" ? v : null);

/** The Message Id of the engine's warning that a deckhouse carries no outfit mass. */
export const OUTFIT_WARNING_ID = "warn-aft-deckhouse-no-outfit";

export interface RuleSetRef {
  id: string;
  version: string;
  status: string;
}

export interface CrewFigures {
  /** The governing manning Rule Set (`crew.rule_set`). */
  ruleSet: RuleSetRef | null;
  /** The crew the Option carries (`crew.crew`), the Rule Set's minimum (`crew.minimum`) and the berths (`crew.berths`). */
  crew: number | null;
  minimum: number | null;
  berths: number | null;
  /** The second Rule Set (`crew.cross_check`): its id, its officers, the governing Rule Set's officers, and the flag. */
  crossCheck: { ruleSet: RuleSetRef | null; otherOfficers: number | null; officers: number | null; difference: number | null; flagged: boolean } | null;
  /** The engine could not work the crew (`crew.error`). */
  error: string | null;
}

export interface GasFigures {
  ventExitAboveDeckM: number | null;
  ventExitAboveKeelM: number | null;
  ventDistanceToAccommodationM: number | null;
  ventDistanceToOpeningsM: number | null;
  ventDistanceTargetM: number | null;
  ventDistanceMet: boolean | null;
  /** `stern_block_mass.gas_fuel_system.items`. */
  fuelItems: { label: string; massT: number | null }[];
}

export interface RoomsFigures {
  berths: number | null;
  totalM2: number | null;
  requiredAreaM2: number | null;
  tiers: number | null;
  rule: string | null;
  ruleNote: string | null;
  settled: boolean;
  rows: { id: string; name: string; count: number | null; areaEachM2: number | null; areaM2: number | null; clause: string }[];
}

export interface SternCrewFigures {
  style: "container_block" | "deckhouse" | "wheelhouse_forward" | null;
  tiers: number | null;
  /** Tiers that hold rooms (`stern_arrangement.room_tiers`, issue #943). */
  roomTiers: number | null;
  /** Tiers that only lift the wheelhouse (`stern_arrangement.access_tiers`). */
  accessTiers: number | null;
  wheelhouse: "fixed" | "elevating" | null;
  /** Why the wheelhouse is of that type (`stern_arrangement.wheelhouse_basis`). */
  wheelhouseBasis: string | null;
  /** The English of the outfit warning of a deckhouse, as it stands in `warnings`. */
  outfitWarning: string | null;
  crew: CrewFigures | null;
  gas: GasFigures | null;
  /** The rooms table of a deckhouse (`rooms`); null for any other style. */
  rooms: RoomsFigures | null;
}

function ruleSetOf(v: unknown): RuleSetRef | null {
  const r = obj(v);
  const id = strOf(r?.id);
  return id ? { id, version: strOf(r?.version) ?? "", status: strOf(r?.status) ?? "" } : null;
}

function crewOf(v: unknown): CrewFigures | null {
  const c = obj(v);
  if (!c) return null;
  const x = obj(c.cross_check);
  return {
    ruleSet: ruleSetOf(c.rule_set),
    crew: numOf(c.crew),
    minimum: numOf(c.minimum),
    berths: numOf(c.berths),
    crossCheck: x
      ? {
          ruleSet: ruleSetOf(x.rule_set),
          otherOfficers: numOf(x.uk_officers),
          officers: numOf(x.liberia_officers),
          difference: numOf(x.difference),
          flagged: x.flagged === true,
        }
      : null,
    error: strOf(c.error),
  };
}

function gasOf(results: Obj): GasFigures | null {
  const inner = obj(results.results);
  if (!inner || numOf(inner.vent_exit_above_keel_m) === null) return null;
  const items = obj(obj(results.stern_block_mass)?.gas_fuel_system)?.items;
  return {
    ventExitAboveDeckM: numOf(inner.vent_exit_above_deck_m),
    ventExitAboveKeelM: numOf(inner.vent_exit_above_keel_m),
    ventDistanceToAccommodationM: numOf(inner.vent_distance_to_accommodation_m),
    ventDistanceToOpeningsM: numOf(inner.vent_distance_to_openings_m),
    ventDistanceTargetM: numOf(inner.vent_mast_distance_target_m),
    ventDistanceMet: typeof inner.vent_mast_distance_met === "boolean" ? inner.vent_mast_distance_met : null,
    fuelItems: (Array.isArray(items) ? items : []).map((i) => ({ label: strOf(obj(i)?.label) ?? "", massT: numOf(obj(i)?.mass_t) })),
  };
}

function roomsOf(v: unknown): RoomsFigures | null {
  const r = obj(v);
  if (!r || !Array.isArray(r.rows)) return null;
  return {
    berths: numOf(r.berths),
    totalM2: numOf(r.total_m2),
    requiredAreaM2: numOf(r.required_area_m2),
    tiers: numOf(r.tiers),
    rule: strOf(r.rule),
    ruleNote: strOf(r.rule_note),
    settled: r.settled === true,
    rows: r.rows.map((x) => {
      const o = obj(x) ?? {};
      return { id: strOf(o.id) ?? "", name: strOf(o.name) ?? "", count: numOf(o.count), areaEachM2: numOf(o.area_each_m2), areaM2: numOf(o.area_m2), clause: strOf(o.clause) ?? "" };
    }),
  };
}

/** The figures of the view; `null` where the Option has no stern block (a barge, a pushed Formation). */
export function sternCrewFigures(
  results: Record<string, unknown> | null | undefined,
  ids?: Record<string, { id: string }> | null,
): SternCrewFigures | null {
  const a = obj(results?.stern_arrangement);
  if (!results || !a) return null;
  const style =
    a.style === "deckhouse" || a.style === "wheelhouse_forward" || a.style === "container_block" ? a.style : null;
  const warnings = Array.isArray(results.warnings) ? (results.warnings as unknown[]) : [];
  const outfit = warnings.find((w): w is string => typeof w === "string" && ids?.[w]?.id === OUTFIT_WARNING_ID) ?? null;
  return {
    style,
    tiers: numOf(a.tiers),
    roomTiers: numOf(a.room_tiers),
    accessTiers: numOf(a.access_tiers),
    wheelhouse: a.wheelhouse === "fixed" || a.wheelhouse === "elevating" ? a.wheelhouse : null,
    wheelhouseBasis: strOf(a.wheelhouse_basis),
    outfitWarning: style === "deckhouse" || style === "wheelhouse_forward" ? outfit : null,
    crew: crewOf(results.crew),
    gas: gasOf(results),
    rooms: roomsOf(results.rooms),
  };
}
