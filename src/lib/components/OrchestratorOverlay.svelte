<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowUp, X } from "@lucide/svelte";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { app, closeOrchestrator, navigateTo, planWork, refreshWorkforce, sendOrchestrator } from "../state/app.svelte.ts";
  import { SUGGESTIONS } from "../orchestrator.ts";
  import Markdown from "./Markdown.svelte";
  import { workforceAvailable } from "../workforce.ts";
  import { clockTime } from "../format.ts";
  import { chatForProject } from "../selectors.ts";
  import type { ChatRef } from "../types.ts";

  let { windowMode = true }: { windowMode?: boolean } = $props();

  const COMPACT = 1;

  let draft = $state("");
  let expanded = $state(false);
  let projectId = $state("core");
  let mode = $state<"chat" | "plan">("chat");
  let planSummary = $state<string | null>(null);
  let streamingText = $state("");
  let input = $state<HTMLTextAreaElement | null>(null);
  let body = $state<HTMLDivElement | null>(null);
  let lastMessage = $state<HTMLElement | null>(null);
  let sequence = 0;

  const live = workforceAvailable();

  let projectChat = $derived(
    app.snapshot ? chatForProject(app.snapshot, projectId, app.chat) : [],
  );
  let visible = $derived(expanded ? projectChat : projectChat.slice(-COMPACT));
  let hiddenCount = $derived(Math.max(0, projectChat.length - COMPACT));

  function projectRef(): ChatRef {
    const project = app.snapshot?.projects.find((candidate) => candidate.id === projectId);
    return {
      kind: "project",
      id: projectId,
      label: project?.key ?? projectId,
    };
  }

  function resizeComposer() {
    if (!input) return;
    input.style.height = "auto";
    input.style.height = `${input.scrollHeight}px`;
  }

  onMount(() => {
    const focus = () => {
      input?.focus();
      void refreshWorkforce();
    };
    focus();
    window.addEventListener("focus", focus);
    window.addEventListener("resize", resizeComposer);
    return () => {
      window.removeEventListener("focus", focus);
      window.removeEventListener("resize", resizeComposer);
    };
  });

  $effect(() => {
    const lastId = visible.at(-1)?.id;
    void lastId;
    void streamingText;
    if (!body) return;
    if (app.pending) {
      body.scrollTop = body.scrollHeight;
      return;
    }
    lastMessage?.scrollIntoView({ block: "start" });
  });

  $effect(() => {
    void draft;
    void mode;
    void input;
    resizeComposer();
  });

  async function plan(goal: string) {
    app.pending = true;
    planSummary = null;
    try {
      const result = await planWork(projectId, goal);
      if (result) {
        const count = result.tasks.length;
        planSummary = `${result.summary} · ${count} ${count === 1 ? "task" : "tasks"} queued`;
        draft = "";
      }
    } finally {
      app.pending = false;
    }
  }

  async function send(value = draft) {
    const text = value.trim();
    if (!text || app.pending || !app.snapshot) return;
    if (mode === "plan") {
      await plan(text);
      return;
    }
    app.chat.push({
      id: `chat-${Date.now()}-${sequence++}`,
      role: "user",
      text,
      at: Date.now(),
      refs: [projectRef()],
    });
    app.pendingChat = [
      {
        id: `pending-${Date.now()}-${sequence++}`,
        role: "user",
        text,
        at: Date.now(),
        refs: [projectRef()],
      },
    ];
    draft = "";
    app.pending = true;
    streamingText = "";

    if (!live) {
      app.pending = false;
      app.pendingChat = [];
      app.workforceError = "Chat requires the desktop app.";
      return;
    }

    try {
      const sent = await sendOrchestrator(projectId, text, (chunk) => {
        streamingText = chunk;
      });
      if (sent) app.pendingChat = [];
    } finally {
      app.pending = false;
      streamingText = "";
    }
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
  <header class="flex h-12 shrink-0 items-center justify-between border-b border-[#232323] px-5">
    <div class="flex items-baseline gap-2.5">
      <span class="text-[11.5px] font-medium uppercase tracking-[0.18em] text-[#ededed]">Micheal</span>
      <span class="text-[9.5px] uppercase tracking-[0.12em] text-[#5f5f5f]">Orchestrator</span>
      {#if live}
        <span class="text-[10px] uppercase tracking-[0.12em] text-[#8a8a8a]">local</span>
      {:else}
        <span class="flex items-center gap-1.5 text-[10px] uppercase tracking-[0.12em] text-[#8a8a8a]">
          <span class="size-1.5 bg-[#39d353]" aria-hidden="true"></span>
          simulated
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-3">
      <span class="font-mono text-[10px] text-[#4d4d4d]">Alt+`</span>
      <button
        onclick={close}
        aria-label="Close orchestrator"
        class="flex min-h-8 items-center gap-1.5 border border-[#232323] px-2.5 text-[11px] text-[#8a8a8a] transition-colors hover:border-[#3d3d3d] hover:text-[#ededed]"
      >
        <X size={13} strokeWidth={2} />
        Esc
      </button>
    </div>
  </header>

  <div bind:this={body} class="min-h-0 flex-1 overflow-y-auto px-5">
    <div class="flex min-h-full flex-col">
      {#each visible as message (message.id)}
      <article bind:this={lastMessage} class="border-b border-[#1a1a1a] py-4 last:border-b-0">
        <div class="flex items-baseline justify-between gap-3">
          <span
            class="text-[10.5px] uppercase tracking-[0.16em] {message.role === 'user' ? 'text-[#ededed]' : 'text-[#8a8a8a]'}"
          >
            {message.role === "user" ? "You" : "Micheal"}
          </span>
          <span class="font-mono text-[10px] text-[#4d4d4d]">{clockTime(message.at)}</span>
        </div>
        {#if message.role === "user"}
          <p class="mt-2 whitespace-pre-wrap break-words text-[15px] leading-[1.65] text-[#ededed]">{message.text}</p>
        {:else}
          <div class="mt-2"><Markdown text={message.text} /></div>
        {/if}
        {#if message.refs.length > 0}
          <div class="mt-2.5 flex flex-wrap gap-2">
            {#each message.refs as ref (ref.kind + ref.id)}
              <button
                onclick={() => openRef(ref)}
                class="min-h-8 border border-[#2b2b2b] px-2.5 font-mono text-[10.5px] text-[#8a8a8a] transition-colors hover:border-[#ededed] hover:text-[#ededed]"
              >
                {ref.label}
              </button>
            {/each}
          </div>
        {/if}
      </article>
    {/each}

    {#if app.pending}
      {#if streamingText.trim()}
        <article class="border-b border-[#1a1a1a] py-4 last:border-b-0" aria-live="polite">
          <div class="flex items-baseline justify-between gap-3">
            <span class="text-[10.5px] uppercase tracking-[0.16em] text-[#8a8a8a]">Micheal</span>
            <span class="font-mono text-[10px] text-[#4d4d4d]">{clockTime(Date.now())}</span>
          </div>
          <div class="mt-2"><Markdown text={streamingText} /></div>
        </article>
      {:else}
        <div class="py-4" aria-live="polite">
          <span class="text-[10.5px] uppercase tracking-[0.16em] text-[#8a8a8a]">Micheal</span>
          <p class="mt-2 flex items-center gap-2 text-[14px] text-[#8a8a8a]">
            <span class="size-1.5 animate-pulse bg-[#ededed]" aria-hidden="true"></span>
            {mode === "plan" ? "Planning" : "Working"}
          </p>
        </div>
      {/if}
    {/if}
    </div>
  </div>

  <div class="shrink-0 border-t border-[#232323] px-5 pb-3 pt-3">
    <div class="mb-2.5 flex items-center gap-2">
      <label class="sr-only" for="orchestrator-project">Project</label>
      <select
        id="orchestrator-project"
        bind:value={projectId}
        disabled={app.pending}
        class="h-9 min-h-9 border border-[#232323] bg-[#0a0a0a] px-2.5 text-[12px] text-[#b4b4b4] focus:border-[#ededed] focus:outline-none"
      >
        {#each app.snapshot?.projects ?? [] as project (project.id)}
          <option value={project.id}>{project.key} · {project.name}</option>
        {/each}
      </select>
      <button
        onclick={() => {
          mode = mode === "plan" ? "chat" : "plan";
          planSummary = null;
        }}
        aria-pressed={mode === "plan"}
        disabled={!live}
        title={live ? "Turn the goal into tasks and start them" : "Planning needs the desktop app"}
        class="min-h-9 border px-3 text-[12px] transition-colors disabled:opacity-40 {mode === 'plan'
          ? 'border-[#ededed] text-[#ededed]'
          : 'border-[#232323] text-[#8a8a8a] hover:border-[#3d3d3d] hover:text-[#ededed]'}"
      >
        Plan &amp; run
      </button>
      {#if mode === "plan"}
        <span class="ml-auto text-[10px] uppercase tracking-[0.12em] text-[#6e6e6e]">goal -> tasks -> run</span>
      {/if}
    </div>

    {#if planSummary}
      <p class="mb-2.5 border-l border-[#ededed] pl-2.5 text-[11.5px] leading-relaxed text-[#b4b4b4]" role="status">
        {planSummary}
      </p>
    {/if}
    {#if app.workforceError}
      <p class="mb-2.5 border-l border-[#c07a7a] pl-2.5 text-[11.5px] leading-relaxed text-[#c07a7a]" role="alert">
        {app.workforceError}
      </p>
    {/if}

    {#if mode === "chat"}
      <div class="mb-2.5 flex flex-wrap gap-2">
        {#each SUGGESTIONS.slice(0, 3) as suggestion (suggestion)}
          <button
            onclick={() => send(suggestion)}
            class="min-h-9 border border-[#232323] px-3 text-[11.5px] text-[#8a8a8a] transition-colors hover:border-[#3d3d3d] hover:text-[#ededed]"
          >
            {suggestion}
          </button>
        {/each}
      </div>
    {/if}

    <div class="flex items-end gap-2 border border-[#3d3d3d] bg-[#0a0a0a] px-3 py-2.5">
      <textarea
        bind:this={input}
        bind:value={draft}
        onkeydown={onComposerKeydown}
        rows={2}
        placeholder={mode === "plan" ? "Describe a goal; ready tasks start automatically…" : "Ask the orchestrator…"}
        aria-label={mode === "plan" ? "Goal for the orchestrator" : "Message the orchestrator"}
        spellCheck={false}
        class="min-h-12 max-h-40 flex-1 resize-none overflow-y-auto bg-transparent text-[15.5px] leading-[1.6] text-[#ededed] placeholder-[#6e6e6e] focus:outline-none"
      ></textarea>
      <button
        onclick={() => send()}
        disabled={!draft.trim() || app.pending}
        aria-label={mode === "plan" ? "Plan and run work" : "Send message"}
        class="flex min-h-11 min-w-[88px] items-center justify-center gap-1.5 border border-[#ededed] bg-[#ededed] px-3 text-[11.5px] font-semibold uppercase tracking-[0.08em] text-black transition-opacity disabled:opacity-30"
      >
        <ArrowUp size={15} strokeWidth={2.2} />
        {mode === "plan" ? "Run plan" : "Send"}
      </button>
    </div>

    <div class="mt-2 flex items-center justify-between text-[10.5px] text-[#6e6e6e]">
      <span>Enter send · Shift+Enter line · Esc close</span>
      <button
        onclick={() => (expanded = !expanded)}
        class="min-h-8 px-1.5 uppercase tracking-[0.08em] transition-colors hover:text-[#ededed]"
      >
        {expanded ? "Compact" : hiddenCount > 0 ? `Full thread · ${hiddenCount}` : "Full thread"}
      </button>
    </div>
  </div>
</div>
