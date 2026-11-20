// The words of the wizard's limit messages. The limits themselves come from the schema (D3).
import type { HardRange, InputSpec, Schema } from "../api";
import { schemaText, t } from "../i18n.svelte";
import type { Problem } from "./wizard.ts";

/** A hard limit as a short phrase: "at least 1 and at most 200". */
export function hardText(h: HardRange): string {
  const parts: string[] = [];
  if (h.gt !== undefined) parts.push(t("app-wizard-over", { value: String(h.gt) }));
  if (h.ge !== undefined) parts.push(t("app-wizard-at-least", { value: String(h.ge) }));
  if (h.lt !== undefined) parts.push(t("app-wizard-under", { value: String(h.lt) }));
  if (h.le !== undefined) parts.push(t("app-wizard-at-most", { value: String(h.le) }));
  return parts.reduce((first, second) => t("app-range-and", { first, second }));
}

/** The label of a schema input, or of a convoy count. */
export function labelOf(schema: Schema, name: string): string {
  if (name === "abreast") return t("app-convoy-abreast");
  if (name === "in_line") return t("app-convoy-in-line");
  const s = schema.inputs.find((i) => i.name === name);
  return s ? schemaText.input(name, s.label) : name;
}

/** What to tell the designer about one problem, in the Display Language. */
export function problemText(schema: Schema, p: Problem): string {
  const label = labelOf(schema, p.input);
  const spec: InputSpec | undefined = schema.inputs.find((i) => i.name === p.input);
  const unit = spec?.unit && spec.unit !== "-" ? ` ${spec.unit}` : "";
  switch (p.kind) {
    case "required":
      return t("app-wizard-required", { label });
    case "number":
      return t("app-wizard-not-a-number", { label });
    case "hard":
      return t("app-wizard-hard-limit", { label, range: p.check?.hard ? hardText(p.check.hard) : "", unit });
    case "soft":
      return t("app-wizard-soft-limit", { label, min: String(p.check?.soft?.min ?? ""), max: String(p.check?.soft?.max ?? ""), unit });
  }
}
