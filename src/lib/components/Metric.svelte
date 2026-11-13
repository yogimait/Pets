<script lang="ts">
  let {
    label,
    value,
    hint,
    tone = "ink",
  }: { label: string; value: string | number; hint?: string; tone?: "ink" | "warn" | "bad" | "ok" } = $props();

  const tones = {
    ink: "text-ink",
    warn: "text-warn",
    bad: "text-bad",
    ok: "text-ok",
  };
</script>

<div class="rounded-panel border border-line-soft bg-surface px-4 py-3.5">
  <p class="text-[10px] uppercase tracking-[0.12em] text-ink-3">{label}</p>
  <p class="mt-2 text-[22px] font-medium leading-none tabular-nums {tones[tone]}">{value}</p>
  {#if hint}<p class="mt-2 truncate text-[11px] text-ink-3">{hint}</p>{/if}
</div>
