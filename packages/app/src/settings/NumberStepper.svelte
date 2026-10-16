<script lang="ts">
  interface Props {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    /** Digits kept after the decimal point (also trims float noise like 1.2000000476). */
    decimals?: number;
    suffix?: string;
    onChange: (value: number) => void;
  }

  let { label, value, min, max, step = 1, decimals = 0, suffix = "", onChange }: Props = $props();

  const round = (n: number) => Number(n.toFixed(decimals));
  const clamp = (n: number) => Math.min(max, Math.max(min, n));

  function set(n: number) {
    if (!Number.isFinite(n)) return;
    const next = round(clamp(n));
    if (next !== round(value)) onChange(next);
  }

  function onInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    // Ignore half-typed values; commit only numbers that are already in range.
    const n = parseFloat(raw);
    if (Number.isFinite(n) && n >= min && n <= max) set(n);
  }

  function onBlur(e: Event) {
    // Snap an out-of-range or empty entry back to the current value.
    (e.currentTarget as HTMLInputElement).value = String(round(value));
  }
</script>

<div class="stepper">
  <span class="label" id="lbl-{label}">{label}</span>
  <div class="control" role="group" aria-labelledby="lbl-{label}">
    <button type="button" aria-label="Decrease {label}" disabled={value <= min} onclick={() => set(value - step)}>−</button>
    <input
      type="number"
      {min}
      {max}
      {step}
      value={round(value)}
      oninput={onInput}
      onblur={onBlur}
      aria-labelledby="lbl-{label}"
    />
    <button type="button" aria-label="Increase {label}" disabled={value >= max} onclick={() => set(value + step)}>+</button>
    {#if suffix}<span class="suffix">{suffix}</span>{/if}
  </div>
</div>

<style>
  .stepper { display: flex; flex-direction: column; gap: 6px; }

  .label { font-size: 12px; color: var(--fg-muted); }

  .control {
    display: inline-flex;
    align-items: stretch;
    width: fit-content;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    overflow: hidden;
  }

  .control:focus-within { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-glow); }

  button {
    width: 32px;
    border: none;
    background: transparent;
    color: var(--fg-muted);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  button:hover:not(:disabled) { background: var(--bg-elev-2); color: var(--fg); }
  button:focus-visible { outline: 2px solid var(--accent-glow); outline-offset: -2px; }
  button:disabled { opacity: 0.35; cursor: default; }

  input[type="number"] {
    width: 56px;
    border: none;
    border-left: 1px solid var(--border);
    border-right: 1px solid var(--border);
    background: transparent;
    color: var(--fg);
    font: inherit;
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    text-align: center;
    padding: 6px 4px;
    outline: none;
    appearance: textfield;
    -moz-appearance: textfield;
  }

  input[type="number"]::-webkit-outer-spin-button,
  input[type="number"]::-webkit-inner-spin-button { appearance: none; margin: 0; }

  .suffix { align-self: center; padding: 0 10px 0 8px; font-size: 12px; color: var(--fg-muted); }
</style>
