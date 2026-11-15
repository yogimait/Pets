<script lang="ts">
  import { inlineSegments, parseMarkdown } from "../markdown.ts";

  let { text }: { text: string } = $props();

  let blocks = $derived(parseMarkdown(text));
</script>

{#snippet inline(value: string)}
  {#each inlineSegments(value) as segment, index (index)}
    {#if segment.kind === "code"}
      <code class="break-words rounded bg-[#171717] px-1 py-0.5 font-mono text-[12.5px] text-[#d4d4d4]">{segment.text}</code>
    {:else if segment.kind === "strong"}
      <strong class="font-semibold text-[#ededed]">{segment.text}</strong>
    {:else}
      {segment.text}
    {/if}
  {/each}
{/snippet}

<div class="space-y-2 text-[15px] leading-[1.65] text-[#c4c4c4]">
  {#each blocks as block, index (index)}
    {#if block.type === "paragraph"}
      <p class="whitespace-pre-wrap break-words">{@render inline(block.text)}</p>
    {:else if block.type === "heading"}
      <p class={block.level <= 2 ? "text-[16px] font-semibold text-[#ededed]" : "text-[15px] font-semibold text-[#ededed]"}>
        {@render inline(block.text)}
      </p>
    {:else if block.type === "code"}
      <pre class="overflow-hidden whitespace-pre-wrap break-words border border-[#232323] bg-[#0a0a0a] p-3 font-mono text-[12.5px] leading-relaxed text-[#d4d4d4]"><code>{block.text}</code></pre>
    {:else if block.type === "list"}
      {#if block.ordered}
        <ol class="list-decimal space-y-1 pl-5">
          {#each block.items as item, itemIndex (itemIndex)}
            <li>{@render inline(item)}</li>
          {/each}
        </ol>
      {:else}
        <ul class="list-disc space-y-1 pl-5">
          {#each block.items as item, itemIndex (itemIndex)}
            <li>{@render inline(item)}</li>
          {/each}
        </ul>
      {/if}
    {:else if block.type === "table"}
      <div class="w-full">
        <table class="w-full table-auto border-collapse text-left text-[13px]">
          <thead>
            <tr>
              {#each block.header as cell, cellIndex (cellIndex)}
                <th class="border border-[#2b2b2b] bg-[#101010] px-2 py-1.5 align-top font-medium text-[#ededed]">
                  {@render inline(cell)}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each block.rows as row, rowIndex (rowIndex)}
              <tr>
                {#each row as cell, cellIndex (cellIndex)}
                  <td class="border border-[#2b2b2b] px-2 py-1.5 align-top break-words">{@render inline(cell)}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else if block.type === "quote"}
      <blockquote class="border-l border-[#3d3d3d] pl-3 text-[#a8a8a8] whitespace-pre-wrap break-words">
        {@render inline(block.text)}
      </blockquote>
    {:else}
      <hr class="border-[#232323]" />
    {/if}
  {/each}
</div>
