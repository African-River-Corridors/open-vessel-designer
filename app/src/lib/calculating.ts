/**
 * The sign "Calculating…" (issue #1082, spec 0017 D13): on from the moment the wait passes `afterMs` until
 * every evaluation sent has come back. Runs can overlap, so a count of runs in flight decides, not a flag:
 * the first run to finish must not hide the sign while another still runs.
 */
export class Calculating {
  private inFlight = 0;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly afterMs: number;
  private readonly onChange: (on: boolean) => void;

  constructor(afterMs: number, onChange: (on: boolean) => void) {
    this.afterMs = afterMs;
    this.onChange = onChange;
  }

  /** Runs in flight now. */
  get running(): number {
    return this.inFlight;
  }

  /** Run `work`; the sign is off again when it ends, whether it returns or throws. */
  async during<T>(work: () => Promise<T>): Promise<T> {
    this.inFlight += 1;
    if (this.inFlight === 1) this.timer = setTimeout(() => this.onChange(true), this.afterMs);
    try {
      return await work();
    } finally {
      this.inFlight -= 1;
      if (this.inFlight === 0) {
        clearTimeout(this.timer);
        this.onChange(false);
      }
    }
  }
}
