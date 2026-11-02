// The one way the app writes a number for reading: `digits` decimals, thousands separators, in a locale.
// `num` (i18n.svelte.ts) and the principal characteristics strip (principal.ts) both call it.
export function formatNumber(value: number, digits: number, maxDigits: number, locale: string): string {
  return value.toLocaleString(locale, { minimumFractionDigits: digits, maximumFractionDigits: maxDigits });
}
