/**
 * The sign "Calculating…" (issue #1082, spec 0017 D13): on from the moment the wait passes `afterMs` until
 * every evaluation sent has come back. Runs can overlap, so a count of runs in flight decides, not a flag:
 * the first run to finish must not hide the sign while another still runs.
 *
 * Issue #1096: on the web an evaluation runs on the page's own thread and blocks it, so the timer cannot fire
 * and the page cannot paint until the run is over. A run the caller expects to be slow (`slow`) turns the sign
 * on at once and waits for `paint` (the page drawing the sign) before the work starts.
 */
export class Calculating {
  private inFlight = 0;
  private shown = false;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private readonly afterMs: number;
  private readonly onChange: (on: boolean) => void;
  private readonly paint: () => Promise<unknown>;

  constructor(afterMs: number, onChange: (on: boolean) => void, paint: () => Promise<unknown> = nextPaint) {
    this.afterMs = afterMs;
    this.onChange = onChange;
    this.paint = paint;
  }

  /** Runs in flight now. */
  get running(): number {
    return this.inFlight;
  }

  private show() {
    this.shown = true;
    this.onChange(true);
  }

  /** Run `work`; the sign is off again when it ends, whether it returns or throws. */
  async during<T>(work: () => Promise<T>, slow = false): Promise<T> {
    this.inFlight += 1;
    if (this.inFlight === 1) this.timer = setTimeout(() => this.show(), this.afterMs);
    try {
      if (slow && !this.shown) {
        clearTimeout(this.timer);
        this.show();
        await this.paint();
      }
      return await work();
    } finally {
      this.inFlight -= 1;
      if (this.inFlight === 0) {
        clearTimeout(this.timer);
        this.shown = false;
        this.onChange(false);
      }
    }
  }
}

/**
 * Resolve once the page has drawn what was just changed: a task queued from an animation frame runs after
 * that frame is painted. A hidden page draws no frames, so a short timer ends the wait there.
 */
export function nextPaint(): Promise<void> {
  return new Promise((resolve) => {
    if (typeof requestAnimationFrame === "function") requestAnimationFrame(() => setTimeout(resolve, 0));
    setTimeout(resolve, 100);
  });
}
