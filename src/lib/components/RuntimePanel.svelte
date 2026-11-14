<script lang="ts">
  import { onDestroy } from "svelte";
  import Panel from "./Panel.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import { app } from "../state/app.svelte.ts";
  import {
    runtimeAbort,
    runtimeAgents,
    runtimeAvailable,
    runtimeCreateSession,
    runtimeInfo,
    runtimeMcp,
    runtimeMessages,
    runtimeModels,
    runtimePermissions,
    runtimePrompt,
    runtimeReplyPermission,
    runtimeStop,
    type RuntimeAgent,
    type RuntimeInfo,
    type RuntimeModel,
    type RuntimePermission,
    type RuntimeSession,
  } from "../runtime.ts";
  import { modelKey, normalizePermissions, normalizeRuntimeMessages, type RuntimeItem } from "../runtime-format.ts";

  const builtinAgents: RuntimeAgent[] = [
    { id: "build", name: "Build", mode: "primary", description: "Default coding agent" },
    { id: "plan", name: "Plan", mode: "primary", description: "Planning without edits" },
    { id: "general", name: "General", mode: "subagent", description: "Broad research and multi-step work" },
    { id: "explore", name: "Explore", mode: "subagent", description: "Read-only code exploration" },
  ];

  let projectId = $state("core");
  let agentId = $state("build");
  let modelValue = $state("opencode-go::iceman-contributor-free");
  let prompt = $state("");
  let info = $state<RuntimeInfo | null>(null);
  let agents = $state<RuntimeAgent[]>([]);
  let models = $state<RuntimeModel[]>([]);
  let mcp = $state<{ name: string; status: { status: string } }[]>([]);
  let session = $state<RuntimeSession | null>(null);
  let items = $state<RuntimeItem[]>([]);
  let permissions = $state<RuntimePermission[]>([]);
  let error = $state("");
  let busy = $state(false);
  let polling = false;
  let timer: ReturnType<typeof setInterval> | undefined;

  let project = $derived(app.snapshot?.projects.find((candidate) => candidate.id === projectId));
  let projectPath = $derived(project?.repoPath ?? "");
  let availableAgents = $derived(agents.length > 0 ? agents.filter((agent) => !agent.hidden) : builtinAgents);
  let modelOptions = $derived(
    models
      .filter((model) => model.enabled && model.capabilities?.tools)
      .sort((a, b) => a.providerID.localeCompare(b.providerID) || a.name.localeCompare(b.name)),
  );
  let selectedModel = $derived(modelOptions.find((model) => modelKey(model.providerID, model.modelID) === modelValue));
  let modelGroups = $derived.by(() => {
    const groups = new Map<string, RuntimeModel[]>();
    for (const model of modelOptions) {
      const list = groups.get(model.providerID) ?? [];
      list.push(model);
      groups.set(model.providerID, list);
    }
    return [...groups.entries()].map(([providerID, entries]) => ({ providerID, entries }));
  });
  let canConnect = $derived(runtimeAvailable() && projectPath.length > 0 && !busy);

  function stopPolling() {
    if (timer) clearInterval(timer);
    timer = undefined;
  }

  async function refresh() {
    if (!session || polling) return;
    polling = true;
    try {
      const [messageData, permissionData] = await Promise.all([
        runtimeMessages(session.id),
        runtimePermissions(session.id),
      ]);
      items = normalizeRuntimeMessages(messageData);
      permissions = normalizePermissions(permissionData);
    } catch (cause) {
      error = String(cause);
    } finally {
      polling = false;
    }
  }

  function startPolling() {
    stopPolling();
    timer = setInterval(refresh, 1200);
  }

  async function connect() {
    if (!canConnect) return;
    busy = true;
    error = "";
    try {
      info = await runtimeInfo(projectPath);
      const [agentData, modelData, mcpData] = await Promise.all([
        runtimeAgents(projectPath),
        runtimeModels(projectPath),
        runtimeMcp(projectPath),
      ]);
      agents = agentData;
      models = modelData;
      mcp = mcpData;
      if (agents.length > 0 && !agents.some((agent) => agent.id === agentId)) {
        agentId = agents[0].id;
      }
      if (!modelOptions.some((model) => modelKey(model.providerID, model.modelID) === modelValue)) {
        const fallback = modelOptions[0];
        if (fallback) modelValue = modelKey(fallback.providerID, fallback.modelID);
      }
    } catch (cause) {
      error = String(cause);
      info = null;
    } finally {
      busy = false;
    }
  }

  async function startSession() {
    const text = prompt.trim();
    if (!text || !info || !selectedModel) return;
    busy = true;
    error = "";
    try {
      session = await runtimeCreateSession({
        project: projectPath,
        title: text.split("\n")[0].slice(0, 80),
        agent: agentId,
        providerId: selectedModel.providerID,
        modelId: selectedModel.modelID,
        permissions: [{ action: "shell", resource: "*", effect: "ask" }],
      });
      prompt = "";
      await runtimePrompt(session.id, text);
      await refresh();
      startPolling();
    } catch (cause) {
      error = String(cause);
    } finally {
      busy = false;
    }
  }

  async function send() {
    const text = prompt.trim();
    if (!text || !session) return;
    prompt = "";
    error = "";
    try {
      await runtimePrompt(session.id, text);
      await refresh();
    } catch (cause) {
      error = String(cause);
    }
  }

  async function abort() {
    if (!session) return;
    try {
      await runtimeAbort(session.id);
      await refresh();
    } catch (cause) {
      error = String(cause);
    }
  }

  async function newSession() {
    if (session) {
      try {
        await runtimeAbort(session.id);
      } catch {
        // Session may already be idle or gone.
      }
    }
    stopPolling();
    session = null;
    items = [];
    permissions = [];
    prompt = "";
  }

  async function reply(permission: RuntimePermission, decision: "once" | "always" | "reject") {
    if (!session) return;
    try {
      await runtimeReplyPermission(session.id, permission.id, decision);
      permissions = permissions.filter((candidate) => candidate.id !== permission.id);
    } catch (cause) {
      error = String(cause);
    }
  }

  async function disconnect() {
    stopPolling();
    session = null;
    items = [];
    permissions = [];
    try {
      await runtimeStop();
    } catch (cause) {
      error = String(cause);
    }
    info = null;
  }

  function onComposerKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      if (session) void send();
      else void startSession();
    }
  }

  onDestroy(stopPolling);
</script>

<Panel title="OpenCode runtime" subtitle="Execution harness, tools, MCP, permissions">
  {#snippet action()}
    <StatusBadge
      status={info ? "active" : "paused"}
      label={info ? `Connected v${info.version}` : runtimeAvailable() ? "Not connected" : "Desktop only"}
    />
  {/snippet}

  <div class="space-y-4">
    <div class="grid gap-2 sm:grid-cols-[1fr_1fr_1.5fr_auto]">
      <select
        bind:value={projectId}
        aria-label="Runtime project"
        class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
      >
        {#each app.snapshot?.projects ?? [] as projectOption (projectOption.id)}
          <option value={projectOption.id}>{projectOption.key} · {projectOption.name}</option>
        {/each}
      </select>

      <select
        bind:value={agentId}
        aria-label="Runtime agent"
        class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
      >
        {#each availableAgents as agent (agent.id)}<option value={agent.id}>{agent.name}</option>{/each}
      </select>

      <select
        bind:value={modelValue}
        aria-label="Runtime model"
        class="h-7 min-w-0 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
      >
        {#each modelGroups as group (group.providerID)}
          <optgroup label={group.providerID}>
            {#each group.entries as model (modelKey(model.providerID, model.modelID))}
              <option value={modelKey(model.providerID, model.modelID)}>{model.name}</option>
            {/each}
          </optgroup>
        {/each}
      </select>

      <button
        onclick={connect}
        disabled={!canConnect}
        class="h-7 rounded-control border border-line-soft bg-surface px-2 text-[10.5px] text-ink-2 transition-colors hover:bg-hover hover:text-ink disabled:opacity-40"
      >
        {busy ? "Connecting" : info ? "Refresh" : "Connect"}
      </button>
    </div>

    <p class="text-[10px] leading-relaxed text-ink-4">
      {projectPath ? `Project: ${projectPath}` : "Select a project."}
      {#if !runtimeAvailable()} Policy and runtime are available only in the desktop app.{/if}
    </p>

    {#if error}
      <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10px] text-bad">{error}</p>
    {/if}

    {#if info}
      <div class="flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-line-soft pt-3 text-[10px] text-ink-3">
        <span class="font-mono">pid {info.pid}</span>
        <span class="font-mono">port {info.port}</span>
        <span>{agents.length > 0 ? `${agents.length} custom agents` : "built-in agents only"}</span>
        <span>{modelOptions.length} tool-capable models</span>
        {#each mcp as server (server.name)}
          <span class="flex items-center gap-1">
            <span class="size-1.5 rounded-full {server.status.status === "connected" ? "bg-ok" : "bg-warn"}"></span>
            <span class="font-mono">{server.name}</span>
          </span>
        {/each}
      </div>
    {/if}

    {#if session}
      <div class="flex items-center justify-between gap-3 border-t border-line-soft pt-3">
        <div class="min-w-0">
          <p class="truncate text-[11px] text-ink">{session.title ?? "Runtime session"}</p>
          <p class="truncate font-mono text-[9px] text-ink-4">{session.id}</p>
        </div>
        <div class="flex shrink-0 gap-1.5">
          <button
            onclick={newSession}
            class="rounded-control border border-line-soft px-2 py-1 text-[10px] text-ink-3 transition-colors hover:text-ink"
          >
            New
          </button>
          <button
            onclick={abort}
            class="rounded-control border border-line-soft px-2 py-1 text-[10px] text-ink-3 transition-colors hover:text-ink"
          >
            Abort
          </button>
          <button
            onclick={disconnect}
            class="rounded-control border border-line-soft px-2 py-1 text-[10px] text-ink-3 transition-colors hover:text-ink"
          >
            Stop
          </button>
        </div>
      </div>

      {#if permissions.length > 0}
        <div class="space-y-2">
          {#each permissions as permission (permission.id)}
            <div class="rounded-control border border-warn/30 bg-warn/5 p-2.5">
              <div class="flex items-center justify-between gap-2">
                <p class="text-[10px] uppercase tracking-[0.1em] text-warn">Permission needed</p>
                <span class="font-mono text-[9px] text-ink-4">{permission.action}</span>
              </div>
              <p class="mt-1 break-words font-mono text-[10px] text-ink-2">{permission.resources.join(" · ")}</p>
              {#if permission.message}<p class="mt-1 text-[10px] text-ink-3">{permission.message}</p>{/if}
              <div class="mt-2 flex flex-wrap gap-1.5">
                <button
                  onclick={() => reply(permission, "once")}
                  class="rounded border border-ok/40 bg-ok/10 px-2 py-0.5 text-[10px] text-ok"
                >
                  Allow once
                </button>
                <button
                  onclick={() => reply(permission, "always")}
                  class="rounded border border-line-soft px-2 py-0.5 text-[10px] text-ink-3 hover:text-ink"
                >
                  Always
                </button>
                <button
                  onclick={() => reply(permission, "reject")}
                  class="rounded border border-bad/40 bg-bad/10 px-2 py-0.5 text-[10px] text-bad"
                >
                  Deny
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <div class="max-h-[420px] space-y-2 overflow-y-auto rounded-control border border-line-soft bg-canvas p-2.5">
        {#each items as item (item.id)}
          {#if item.kind === "user"}
            <div class="flex justify-end">
              <p class="max-w-[88%] rounded-control border border-line bg-raised px-2.5 py-1.5 text-[11.5px] text-ink">{item.text}</p>
            </div>
          {:else if item.kind === "assistant"}
            <p class="whitespace-pre-wrap text-[11.5px] leading-relaxed text-ink-2">{item.text}</p>
          {:else if item.kind === "tool"}
            <div class="rounded-control border border-line-soft bg-surface p-2">
              <div class="flex items-center justify-between gap-2">
                <span class="font-mono text-[10px] text-ink-2">{item.toolName}</span>
                <StatusBadge status={item.toolStatus ?? "unknown"} />
              </div>
              {#if item.input}<pre class="mt-1 max-h-24 overflow-auto whitespace-pre-wrap font-mono text-[9px] text-ink-4">{item.input}</pre>{/if}
              {#if item.output}<pre class="mt-1 max-h-32 overflow-auto whitespace-pre-wrap font-mono text-[9.5px] text-ink-3">{item.output}</pre>{/if}
            </div>
          {/if}
        {/each}
        {#if items.length === 0}
          <p class="py-4 text-center text-[10.5px] text-ink-4">No messages yet.</p>
        {/if}
      </div>

      <div class="flex items-end gap-2 rounded-control border border-line bg-canvas p-1.5 focus-within:border-accent/40">
        <textarea
          bind:value={prompt}
          onkeydown={onComposerKeydown}
          rows={1}
          placeholder="Send a follow-up instruction…"
          aria-label="Runtime prompt"
          class="max-h-24 min-h-7 flex-1 resize-none bg-transparent px-1.5 py-1 text-[11.5px] text-ink placeholder:text-ink-4 focus:outline-none"
        ></textarea>
        <button
          onclick={send}
          disabled={!prompt.trim() || !session}
          class="rounded-control border border-line-soft bg-raised px-2 py-1 text-[10px] text-ink-2 disabled:opacity-40"
        >
          Send
        </button>
      </div>
    {:else}
      <div class="flex items-end gap-2 rounded-control border border-line bg-canvas p-1.5 focus-within:border-accent/40">
        <textarea
          bind:value={prompt}
          onkeydown={onComposerKeydown}
          rows={2}
          placeholder="Start a real OpenCode session with this instruction…"
          aria-label="Runtime task prompt"
          class="max-h-32 min-h-12 flex-1 resize-none bg-transparent px-1.5 py-1 text-[11.5px] text-ink placeholder:text-ink-4 focus:outline-none"
        ></textarea>
        <button
          onclick={startSession}
          disabled={!info || !prompt.trim() || busy || !selectedModel}
          class="rounded-control border border-line-soft bg-raised px-2 py-1 text-[10px] text-ink-2 disabled:opacity-40"
        >
          Start session
        </button>
      </div>
      <p class="text-[9.5px] text-ink-4">
        Runs a real OpenCode session. Shell commands ask for approval. Session history is linked to this panel for now; task linking comes next.
      </p>
    {/if}
  </div>
</Panel>
