<script lang="ts">
  let { status, label }: { status: string; label?: string } = $props();

  const tones: Record<string, string> = {
    active: "text-ok",
    running: "text-ok",
    done: "text-ok",
    approved: "text-ok",
    review: "text-warn",
    pending: "text-warn",
    partial: "text-warn",
    blocked: "text-bad",
    rejected: "text-bad",
    offline: "text-bad",
    idle: "text-ink-3",
    queued: "text-ink-3",
    paused: "text-ink-3",
    archived: "text-ink-4",
  };

  let tone = $derived(tones[status] ?? "text-ink-3");
  let text = $derived(label ?? status.charAt(0).toUpperCase() + status.slice(1));
</script>

<span
  class="inline-flex items-center gap-1.5 rounded border border-line-soft bg-raised px-1.5 py-[1px] text-[10.5px] text-ink-2 {tone}"
>
  <span class="size-1.5 rounded-full bg-current" aria-hidden="true"></span>
  {text}
</span>
