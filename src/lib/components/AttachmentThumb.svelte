<script lang="ts">
  import { workforceAttachmentData } from "../workforce.ts";
  import type { ChatAttachment } from "../types.ts";

  let { attachment }: { attachment: ChatAttachment } = $props();

  let src = $state("");

  $effect(() => {
    let cancelled = false;
    void workforceAttachmentData(attachment.path)
      .then((result) => {
        if (!cancelled) src = `data:${attachment.mime};base64,${result.data}`;
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });
</script>

{#if src}
  <img
    src={src}
    alt={attachment.name}
    title={attachment.name}
    class="h-14 max-w-[120px] border border-[#2b2b2b] object-cover"
  />
{/if}
