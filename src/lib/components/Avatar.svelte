<script lang="ts">
  import { initials } from "../format.ts";
  import { petForAgent } from "../pets.ts";

  let {
    name,
    agentId,
    size = "md",
    accent = false,
  }: { name: string; agentId?: string; size?: "xs" | "sm" | "md"; accent?: boolean } = $props();

  let pet = $derived(agentId ? petForAgent(agentId) : undefined);

  const sizes = {
    xs: "size-5 text-[9px]",
    sm: "size-6 text-[10px]",
    md: "size-8 text-[11px]",
  };
</script>

<span
  class="grid shrink-0 place-items-center overflow-hidden rounded-control border bg-raised font-sans font-medium tracking-wide {sizes[size]} {accent
    ? 'border-accent/40 text-accent'
    : 'border-line-soft text-ink-2'}"
  aria-hidden="true"
>
  {#if pet}
    <img
      src={`/pets/${pet.sprite}.png`}
      alt=""
      class="h-full w-full object-contain [image-rendering:pixelated]"
      draggable="false"
    />
  {:else}
    {initials(name)}
  {/if}
</span>
