// Issue #924: the damage run of the web build. The page's wasm session runs in the page's main thread, and a
// damage run there would freeze it. This worker has a wasm session of its own: it is sent the Design, the
// Defaults Profile and the Option's name, runs the damage cases, posts how far it is, then posts the run.
// The page cancels with `worker.terminate()`. The worker holds no calculation of its own.
import init, { Session } from "../../../wasm-pkg/ovd_web.js";
import wasmUrl from "../../../wasm-pkg/ovd_web_bg.wasm?url";

export interface DamageJob {
  name: string;
  design: string;
  profile: string;
  option: string;
}
export type DamageMessage =
  | { kind: "progress"; done: number; total: number }
  | { kind: "done"; run: string }
  | { kind: "failed"; message: string };

const post = (m: DamageMessage) => (self as unknown as { postMessage(m: DamageMessage): void }).postMessage(m);

self.onmessage = async (e: MessageEvent<DamageJob>) => {
  const job = e.data;
  try {
    await init({ module_or_path: wasmUrl });
    const session = new Session();
    session.open_text(job.name, job.design);
    session.load_profile_text(job.profile);
    const run = session.run_damage(job.option, (done: number, total: number) => post({ kind: "progress", done, total }));
    post({ kind: "done", run });
  } catch (err) {
    post({ kind: "failed", message: String(err) });
  }
};
