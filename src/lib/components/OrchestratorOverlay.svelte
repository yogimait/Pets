<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ArrowUp, X } from "@lucide/svelte";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { app, closeOrchestrator, navigateTo } from "../state/app.svelte.ts";
  import { SUGGESTIONS, respond } from "../orchestrator.ts";
  import { clockTime } from "../format.ts";
  import type { ChatRef } from "../types.ts";

  let { windowMode = true }: { windowMode?: boolean } = $props();

  const COMPACT = 2;

  let draft = $state("");
  let expanded = $state(false);
  let input = $state<HTMLTextAreaElement | null>(null);
  let body = $state<HTMLDivElement | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let sequence = 0;

  let visible = $derived(expanded ? app.chat : app.chat.slice(-COMPACT));
  let hiddenCount = $derived(Math.max(0, app.chat.length - COMPACT));

  onMount(() => {
    const focus = () => input?.focus();
    focus();
    window.addEventListener("focus", focus);
    return () => window.removeEventListener("focus", focus);
  });

  onDestroy(() => {
    if (timer) clearTimeout(timer);
  });

  $effect(() => {
    const length = visible.length;
    const pending = app.pending;
    void length;
    void pending;
    if (body) body.scrollTop = body.scrollHeight;
  });

  function send(value = draft) {
    const text = value.trim();
    if (!text || app.pending || !app.snapshot) return;
    const snapshot = app.snapshot;
    app.chat.push({
      id: `chat-${Date.now()}-${sequence++}`,
      role: "user",
      text,
      at: Date.now(),
      refs: [],
    });
    draft = "";
    app.pending = true;
    timer = setTimeout(() => {
      const reply = respond(text, snapshot);
      app.chat.push({
        id: `chat-${Date.now()}-${sequence++}`,
        role: "orchestrator",
        text: reply.text,
        at: Date.now(),
        refs: reply.refs,
      });
      app.pending = false;
    }, 650);
  }

  function onComposerKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  async function openRef(ref: ChatRef) {
    if (isTauri()) {
      try {
        await invoke("open_record", { kind: ref.kind, id: ref.id });
        return;
      } catch (error) {
        console.warn("record navigation failed", error);
      }
    }
    app.overlayOpen = false;
    navigateTo(ref.kind, ref.id);
  }

  function close() {
    closeOrchestrator();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") close();
  }}
/>

<div class="flex h-full flex-col bg-[#050505] text-[#ededed] {windowMode ? 'border border-[#3d3d3d]' : ''}">
  <header class="flex h-10 shrink-0 items-center justify-between border-b border-[#232323] px-4">
    <div class="flex items-baseline gap-2.5">
      <span class="text-[10px] font-medium uppercase tracking-[0.18em] text-[#ededed]">Orchestrator</span>
      <span class="flex items-center gap-1.5 text-[9px] uppercase tracking-[0.12em] text-[#8a8a8a]">
        <span class="size-1.5 bg-[#39d353]" aria-hidden="true"></span>
        simulated
      </span>
    </div>
    <div class="flex items-center gap-3">
      <span class="font-mono text-[9px] text-[#4d4d4d]">Alt+`</span>
      <button
        onclick={close}
        aria-label="Close orchestrator"
        class="flex items-center gap-1 border border-[#232323] px-1.5 py-0.5 text-[10px] text-[#8a8a8a] transition-colors hover:border-[#3d3d3d] hover:text-[#ededed]"
      >
        <X size={11} strokeWidth={2} />
        Esc
      </button>
    </div>
  </header>

  <div bind:this={body} class="min-h-0 flex-1 overflow-y-auto px-4">
    <div class={expanded ? "" : "flex min-h-full flex-col justify-end"}>
      {#each visible as message (message.id)}
      <article class="border-b border-[#1a1a1a] py-3 last:border-b-0">
        <div class="flex items-baseline justify-between gap-3">
          <span
            class="text-[9px] uppercase tracking-[0.16em] {message.role === 'user' ? 'text-[#ededed]' : 'text-[#8a8a8a]'}"
          >
            {message.role === "user" ? "You" : "Orchestrator"}
          </span>
          <span class="font-mono text-[9px] text-[#4d4d4d]">{clockTime(message.at)}</span>
        </div>
        <p class="mt-1.5 text-[13px] leading-relaxed {message.role === 'user' ? 'text-[#ededed]' : 'text-[#b4b4b4]'}">
          {message.text}
        </p>
        {#if message.refs.length > 0}
          <div class="mt-2 flex flex-wrap gap-1.5">
            {#each message.refs as ref (ref.kind + ref.id)}
              <button
                onclick={() => openRef(ref)}
                class="border border-[#2b2b2b] px-1.5 py-0.5 font-mono text-[9.5px] text-[#8a8a8a] transition-colors hover:border-[#ededed] hover:text-[#ededed]"
              >
                {ref.label}
              </button>
            {/each}
          </div>
        {/if}
      </article>
    {/each}

    {#if app.pending}
      <div class="py-3" aria-live="polite">
        <span class="text-[9px] uppercase tracking-[0.16em] text-[#8a8a8a]">Orchestrator</span>
        <p class="mt-1.5 text-[13px] text-[#6e6e6e]">thinking…</p>
      </div>
    {/if}
    </div>
  </div>

  {#if hiddenCount > 0 && !expanded}
    <button
      onclick={() => (expanded = true)}
      class="shrink-0 border-t border-[#1a1a1a] px-4 py-1.5 text-left text-[9.5px] uppercase tracking-[0.12em] text-[#6e6e6e] transition-colors hover:text-[#ededed]"
    >
      Show {hiddenCount} earlier {hiddenCount === 1 ? "message" : "messages"}
    </button>
  {/if}

  <div class="shrink-0 border-t border-[#232323] px-4 pb-2.5 pt-2">
    <div class="mb-2 flex flex-wrap gap-1.5">
      {#each SUGGESTIONS.slice(0, 3) as suggestion (suggestion)}
        <button
          onclick={() => send(suggestion)}
          class="border border-[#232323] px-2 py-0.5 text-[10px] text-[#8a8a8a] transition-colors hover:border-[#3d3d3d] hover:text-[#ededed]"
        >
          {suggestion}
        </button>
      {/each}
    </div>

    <div class="flex items-end gap-2 border border-[#3d3d3d] bg-[#0a0a0a] px-3 py-2">
      <textarea
        bind:this={input}
        bind:value={draft}
        onkeydown={onComposerKeydown}
        rows={1}
        placeholder="Ask the orchestrator…"
        aria-label="Message the orchestrator"
        spellCheck={false}
        class="max-h-24 min-h-6 flex-1 resize-none bg-transparent text-[15px] leading-relaxed text-[#ededed] placeholder-[#6e6e6e] focus:outline-none"
      ></textarea>
      <button
        onclick={() => send()}
        disabled={!draft.trim() || app.pending}
        aria-label="Send message"
        class="flex items-center gap-1 border border-[#ededed] bg-[#ededed] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.08em] text-black transition-opacity disabled:opacity-30"
      >
        <ArrowUp size={12} strokeWidth={2.2} />
        Send
      </button>
    </div>

    <div class="mt-1.5 flex items-center justify-between text-[9.5px] text-[#6e6e6e]">
      <span>Enter send · Shift+Enter line · Esc close</span>
      <button
        onclick={() => (expanded = !expanded)}
        class="uppercase tracking-[0.08em] transition-colors hover:text-[#ededed]"
      >
        {expanded ? "Compact" : "Full thread"}
      </button>
    </div>
  </div>
</div>
