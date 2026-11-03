<script lang="ts">
  import { num, say, t, type TextIds } from "../i18n.svelte";
  import { OUTFIT_WARNING_ID, sternCrewFigures } from "../sternCrew";

  // The stern of a self-propelled Option and its crew, as the engine gives them (`stern_arrangement`,
  // `crew`, the vent mast keys and `stern_block_mass.gas_fuel_system` of the results, issue #887). The
  // app adds nothing up. The full value of a number is on hover. Concept and preliminary design only:
  // not a Safe Manning Document.
  let { results, ids }: { results: Record<string, unknown> | null | undefined; ids: TextIds | null } = $props();
  const f = $derived(sternCrewFigures(results, ids));
  const n = (v: number | null | undefined, dp: number) => (v === null || v === undefined ? "–" : num(v, dp));
  const full = (v: number | null | undefined) => (v === null || v === undefined ? "" : String(v));
  const rule = (r: { id: string; version: string; status: string } | null) => (r ? `${r.id} ${r.version} (${r.status})` : "–");
</script>

<div class="stern" data-testid="stern-crew">
  {#if !f}
    <p class="note" data-testid="stern-none">{t("app-stern-none")}</p>
  {:else}
    <h4>{t("app-stern-arrangement")}</h4>
    <table>
      <tbody>
        <tr data-row="style"><th>{t("app-stern-style")}</th><td data-cell="value">{f.style ? t(`app-stern-style-${f.style}`) : "–"}</td></tr>
        {#if f.style === "deckhouse"}
          <tr data-row="tiers"><th>{t("app-stern-tiers")}</th><td data-cell="value">{n(f.tiers, 0)}</td></tr>
        {/if}
        <tr data-row="wheelhouse">
          <th>{t("app-stern-wheelhouse")}</th>
          <td data-cell="value">{f.wheelhouse ? t(`app-stern-wheelhouse-${f.wheelhouse}`) : "–"}{#if f.wheelhouseBasis}<span class="why"> ({t(`app-stern-wheelhouse-why-${f.wheelhouseBasis}`)})</span>{/if}</td>
        </tr>
      </tbody>
    </table>
    {#if f.outfitWarning}
      <!-- The engine's own warning, found by its Message Id; there is no fallback text, so a run that lacks it shows nothing. -->
      <p class="warn" data-testid="stern-outfit-warning" data-warning-id={OUTFIT_WARNING_ID}>{say(f.outfitWarning, ids)}</p>
    {/if}

    <h4>{t("app-stern-crew")}</h4>
    {#if !f.crew}
      <p class="note" data-testid="stern-crew-none">{t("app-stern-crew-none")}</p>
    {:else if f.crew.error}
      <p class="note" data-testid="stern-crew-error">{f.crew.error}</p>
    {:else}
      <table data-testid="stern-crew-table">
        <tbody>
          <tr data-row="rule-set"><th>{t("app-stern-rule-set")}</th><td data-cell="value">{rule(f.crew.ruleSet)}</td></tr>
          <tr data-row="crew"><th>{t("app-stern-crew-count")}</th><td data-cell="value" class="num" title={full(f.crew.crew)}>{n(f.crew.crew, 0)}</td></tr>
          <tr data-row="minimum"><th>{t("app-stern-minimum")}</th><td data-cell="value" class="num" title={full(f.crew.minimum)}>{n(f.crew.minimum, 0)}</td></tr>
          <tr data-row="berths"><th>{t("app-stern-berths")}</th><td data-cell="value" class="num" title={full(f.crew.berths)}>{n(f.crew.berths, 0)}</td></tr>
          {#if f.crew.crossCheck}
            <tr data-row="cross-check">
              <th>{t("app-stern-cross-check")}</th>
              <td data-cell="value">{rule(f.crew.crossCheck.ruleSet)}: {t("app-stern-cross-officers", { other: n(f.crew.crossCheck.otherOfficers, 0), own: n(f.crew.crossCheck.officers, 0) })}; {f.crew.crossCheck.flagged ? t("app-stern-flagged") : t("app-stern-not-flagged")}</td>
            </tr>
          {/if}
        </tbody>
      </table>
    {/if}

    {#if f.gas}
      <h4>{t("app-stern-gas")}</h4>
      <table data-testid="stern-gas-table">
        <tbody>
          <tr data-row="vent-deck"><th>{t("app-stern-vent-deck")}<span class="unit">&nbsp;(m)</span></th><td data-cell="value" class="num" title={full(f.gas.ventExitAboveDeckM)}>{n(f.gas.ventExitAboveDeckM, 2)}</td></tr>
          <tr data-row="vent-keel"><th>{t("app-stern-vent-keel")}<span class="unit">&nbsp;(m)</span></th><td data-cell="value" class="num" title={full(f.gas.ventExitAboveKeelM)}>{n(f.gas.ventExitAboveKeelM, 2)}</td></tr>
          <tr data-row="vent-accommodation"><th>{t("app-stern-vent-accommodation")}<span class="unit">&nbsp;(m)</span></th><td data-cell="value" class="num" title={full(f.gas.ventDistanceToAccommodationM)}>{n(f.gas.ventDistanceToAccommodationM, 2)}</td></tr>
          <tr data-row="vent-openings"><th>{t("app-stern-vent-openings")}<span class="unit">&nbsp;(m)</span></th><td data-cell="value" class="num" title={full(f.gas.ventDistanceToOpeningsM)}>{n(f.gas.ventDistanceToOpeningsM, 2)}</td></tr>
          <tr data-row="vent-target"><th>{t("app-stern-vent-target")}<span class="unit">&nbsp;(m)</span></th><td data-cell="value" class="num" title={full(f.gas.ventDistanceTargetM)}>{n(f.gas.ventDistanceTargetM, 2)}{#if f.gas.ventDistanceMet !== null}<span class="why"> ({f.gas.ventDistanceMet ? t("app-stern-met") : t("app-stern-not-met")})</span>{/if}</td></tr>
        </tbody>
      </table>
      {#if f.gas.fuelItems.length > 0}
        <table data-testid="stern-fuel-items">
          <thead><tr><th>{t("app-stern-fuel-item")}</th><th class="num">{t("app-stern-fuel-mass")}<span class="unit">&nbsp;(t)</span></th></tr></thead>
          <tbody>
            {#each f.gas.fuelItems as i (i.label)}
              <tr data-row="fuel-item"><th>{say(i.label, ids)}</th><td data-cell="value" class="num" title={full(i.massT)}>{n(i.massT, 2)}</td></tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}

    <h4>{t("app-stern-rooms")}</h4>
    {#if f.rooms}
      <table data-testid="stern-rooms">
        <thead>
          <tr>
            <th>{t("app-stern-room")}</th>
            <th class="num">{t("app-stern-room-count")}</th>
            <th class="num">{t("app-stern-room-each")}<span class="unit">&nbsp;(m²)</span></th>
            <th class="num">{t("app-stern-room-area")}<span class="unit">&nbsp;(m²)</span></th>
            <th>{t("app-stern-room-clause")}</th>
          </tr>
        </thead>
        <tbody>
          {#each f.rooms.rows as r (r.id)}
            <tr data-row="room">
              <th>{say(r.name, ids)}</th>
              <td class="num" title={full(r.count)}>{n(r.count, 0)}</td>
              <td class="num" title={full(r.areaEachM2)}>{n(r.areaEachM2, 2)}</td>
              <td class="num" title={full(r.areaM2)}>{n(r.areaM2, 2)}</td>
              <td class="why">{r.clause}</td>
            </tr>
          {/each}
          <tr data-row="rooms-total"><th>{t("app-stern-rooms-total")}</th><td></td><td></td><td class="num" title={full(f.rooms.totalM2)}>{n(f.rooms.totalM2, 2)}</td><td></td></tr>
        </tbody>
      </table>
      {#if f.rooms.ruleNote}<p class="note">{say(f.rooms.ruleNote, ids)}</p>{/if}
      {#if !f.rooms.settled}<p class="note" data-testid="stern-rooms-unsettled">{t("app-stern-rooms-unsettled")}</p>{/if}
    {:else}
      <p class="note" data-testid="stern-rooms">{t("app-stern-rooms-none")}</p>
    {/if}
    <p class="note" data-testid="stern-drawing-later">{t("app-stern-drawing-later")}</p>
  {/if}
</div>

<style>
  .stern { height: 100%; overflow: auto; }
  h4 { margin: 10px 0 4px; font-size: var(--fs-13); color: var(--ink-2); }
  h4:first-child { margin-top: 0; }
  table { border-collapse: collapse; font-size: 13px; margin-bottom: 4px; }
  th { text-align: left; font-weight: normal; padding: 3px 14px 3px 0; white-space: nowrap; }
  thead th { color: var(--ink-3); }
  td { padding: 3px 8px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit, .why { color: var(--ink-3); }
  .note { color: var(--ink-3); margin: 4px 0; }
  .warn { color: var(--warn); background: var(--warn-soft); border: var(--lw-thin) solid var(--warn); border-radius: var(--radius); padding: 4px 8px; margin: 6px 0; max-width: 70ch; }
</style>
