<script lang="ts">
  import { initials } from "../format.ts";

  let {
    name,
    size = "md",
    accent = false,
  }: { name: string; size?: "xs" | "sm" | "md"; accent?: boolean } = $props();

  const sizes = {
    xs: "size-5 text-[9px]",
    sm: "size-6 text-[10px]",
    md: "size-8 text-[11px]",
  };
</script>

<span
  class="grid shrink-0 place-items-center rounded-control border bg-raised font-sans font-medium tracking-wide {sizes[size]} {accent
    ? 'border-accent/40 text-accent'
    : 'border-line-soft text-ink-2'}"
  aria-hidden="true">{initials(name)}</span
>
