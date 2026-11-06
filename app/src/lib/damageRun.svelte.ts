// Issue #924: the damage run the app has under way, if any: what the message bar and the Damage view show.
// Display state only: the engine runs the cases and says how far it is; the time left is worked out from that
// (see damage.ts).
import { loadSecondsPerCase } from "./damage";

export interface ActiveDamageRun {
  option: string;
  done: number;
  total: number;
  /** performance.now() at the start, and at the last progress event. */
  startedAt: number;
  lastAt: number;
}

export const damageRun = $state<{
  active: ActiveDamageRun | null;
  /** The time per case measured by the last finished run on this machine (seconds), or null. */
  secondsPerCase: number | null;
}>({ active: null, secondsPerCase: loadSecondsPerCase() });
