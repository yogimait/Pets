<script lang="ts">
  import { onMount } from "svelte";
  import Panel from "../components/Panel.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import { autostartEnabled, setAutostart } from "../autostart.ts";
  import Avatar from "../components/Avatar.svelte";
  import RuntimePanel from "../components/RuntimePanel.svelte";
  import { opencodeAgentId } from "../selectors.ts";
  import { app, loadPolicy, reloadPolicy } from "../state/app.svelte.ts";
  import { notify } from "../notifications.ts";
  import { workforceAvailable } from "../workforce.ts";
  import { fullTimestamp } from "../format.ts";
  import type { Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const live = workforceAvailable();
  let autostart = $state(false);
  let autostartBusy = $state(false);

  onMount(() => {
    if (!live) return;
    void autostartEnabled()
      .then((enabled) => (autostart = enabled))
      .catch((error) => console.warn("autostart status unavailable", error));
  });

  async function toggleAutostart() {
    autostartBusy = true;
    try {
      autostart = await setAutostart(!autostart);
    } catch (error) {
      console.warn("autostart toggle unavailable", error);
    } finally {
      autostartBusy = false;
    }
  }

  function changePolicyProject(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    loadPolicy(select.value);
  }

  const tokens = [
    { name: "Canvas", value: "#0a0b0d" },
    { name: "Surface", value: "#111316" },
    { name: "Raised", value: "#16181c" },
    { name: "Line", value: "#23262b" },
    { name: "Ink", value: "#e6e8eb" },
    { name: "Muted", value: "#71777d" },
    { name: "Accent", value: "#8ca6c0" },
  ];

</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Settings</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      {live
        ? "Workforce state is stored locally. Models run only when you start a task or a session."
        : "Browser preview has no backend. Open the desktop app for real workspace data."}
    </p>
  </div>

  <Panel title="Data source" subtitle={live ? "Local SQLite" : "Browser preview"}>
    <div class="flex items-start gap-3 rounded-control border border-line-soft bg-raised p-3">
      <span class="mt-1.5 size-1.5 shrink-0 rounded-full {live ? 'bg-ok' : 'bg-warn'}" aria-hidden="true"></span>
      <div>
        <p class="text-[11.5px] text-ink">
          {live ? "Local workspace database" : "Read-only browser preview"}
        </p>
        <p class="mt-1 text-[10.5px] leading-relaxed text-ink-3">
          {live
            ? "Agents, projects, tasks, activity, memory, proposals, and the orchestrator thread live in a SQLite database in the app data directory. New workspaces start empty. Every action returns the stored snapshot."
            : "No backend is attached. Open the desktop app to create projects, run agents, or change data."}
        </p>
      </div>
    </div>
    <ul class="mt-3 space-y-1.5 text-[10.5px] leading-relaxed text-ink-3">
      <li>
        <span class="text-ink-4">Data seam</span> ·
        <span class="font-mono text-ink-2">src/lib/data/provider.ts</span> routes reads; workforce.ts owns every write command.
      </li>
      <li>
        <span class="text-ink-4">Orchestrator</span> ·
        {live
          ? "the overlay sends the prompt to the local backend and shows the stored reply. Plan mode creates tasks."
          : "chat and Plan require the desktop app."}
      </li>
      <li>
        <span class="text-ink-4">Persistence</span> ·
        {live
          ? "workforce state survives a restart. OpenCode sessions live outside the app and are linked by id."
          : "app state is not persisted. OpenCode sessions live outside the app."}
      </li>
      <li>
        <span class="text-ink-4">Execution</span> ·
        {live
          ? "no model runs until you send, plan, or start a session. Idle time costs nothing."
          : "the browser preview cannot start models."}
      </li>
    </ul>
  </Panel>

  <Panel title="Startup" subtitle="Open with Windows">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <p class="text-[11.5px] text-ink">Open Agent Bots when I sign in</p>
        <p class="mt-1 max-w-[560px] text-[10.5px] leading-relaxed text-ink-3">
          {live
            ? "The first app launch enables login startup unless you turn it off here. Windows registers the installed executable in your user startup list."
            : "Startup registration requires the desktop app."}
        </p>
      </div>
      <label class="flex items-center gap-2 text-[11px] text-ink-2">
        <input
          type="checkbox"
          checked={autostart}
          disabled={!live || autostartBusy}
          onchange={toggleAutostart}
          class="size-3.5 accent-[#8ca6c0]"
        />
        <span>{autostart ? "Enabled" : "Disabled"}</span>
      </label>
    </div>
  </Panel>

  <Panel title="Notifications" subtitle="Windows desktop alerts">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <p class="text-[11.5px] text-ink">Final project notifications</p>
        <p class="mt-1 max-w-[560px] text-[10.5px] leading-relaxed text-ink-3">
          {live
            ? "The first notification asks Windows for permission. If alerts are blocked, open Windows Settings > System > Notifications and allow Agent Bots. Final project complete/stopped events are always sent here; the Alerts bell stays in-app."
            : "Desktop notifications require the desktop app."}
        </p>
      </div>
      {#if live}
        <button
          onclick={() => void notify("Agent Bots", "Desktop notifications are enabled.")}
          class="rounded-control border border-line-soft bg-surface px-3 py-1.5 text-[10.5px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
        >
          Send test notification
        </button>
      {/if}
    </div>
  </Panel>

  <Panel title="Global policy" subtitle="Loaded verbatim from the OpenCode config">
    {#snippet action()}
      <button
        onclick={reloadPolicy}
        disabled={app.policyLoading}
        class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink disabled:opacity-40"
      >
        {app.policyLoading ? "Loading" : "Reload"}
      </button>
    {/snippet}
    <div class="space-y-3">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <p class="break-all font-mono text-[10px] text-ink-2">{app.policy?.global.path ?? "Loading policy…"}</p>
          <p class="mt-1 font-mono text-[9.5px] text-ink-4">
            {app.policy
              ? `${app.policy.global.bytes} bytes · ${app.policy.global.hash || "no hash"} · loaded ${fullTimestamp(app.policy.loadedAt)}`
              : "No policy loaded."}
          </p>
        </div>
        <StatusBadge
          status={app.policy?.global.exists ? "active" : "paused"}
          label={app.policy?.global.exists ? "Loaded" : "Missing"}
        />
      </div>

      <div class="flex flex-wrap items-center gap-2 border-t border-line-soft pt-3">
        <select
          value={app.policyProjectId ?? "core"}
          onchange={changePolicyProject}
          aria-label="Policy project"
          class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
        >
          {#each snapshot.projects as project (project.id)}
            <option value={project.id}>{project.key} · {project.name}</option>
          {/each}
        </select>
        <span class="min-w-0 flex-1 truncate font-mono text-[9.5px] text-ink-4">
          {app.policy?.project?.path ?? "No project AGENTS.md loaded"}
        </span>
        <StatusBadge
          status={app.policy?.project?.exists ? "active" : "paused"}
          label={app.policy?.project?.exists ? "Project policy" : "No project policy"}
        />
      </div>

      {#if app.policyError}
        <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10px] text-bad">{app.policyError}</p>
      {:else if app.policy?.global.error}
        <p class="rounded-control border border-warn/30 bg-warn/5 px-2.5 py-2 text-[10px] text-warn">
          {app.policy.global.error}
        </p>
      {/if}

      <p class="text-[10px] leading-relaxed text-ink-4">
        The backend loads this file before every real prompt and blocks a run when it or the project AGENTS.md is missing. The project file may add or tighten rules; it never replaces the global policy.
      </p>
    </div>
  </Panel>

  <RuntimePanel />

  <Panel title="Model execution" subtitle="Resolved by the local backend">
    <div class="grid gap-2 sm:grid-cols-3">
      <div class="rounded-control border border-line-soft bg-raised px-3 py-2">
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Selection</p>
        <p class="mt-1 text-[11.5px] text-ink">OpenCode agent</p>
      </div>
      <div class="rounded-control border border-line-soft bg-raised px-3 py-2">
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Override</p>
        <p class="mt-1 truncate font-mono text-[10px] text-ink-2">AGENT_BOTS_MODEL</p>
      </div>
      <div class="rounded-control border border-line-soft bg-raised px-3 py-2">
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Credentials</p>
        <p class="mt-1 text-[11.5px] text-ink">Owned by OpenCode</p>
      </div>
    </div>
    <p class="mt-3 text-[10.5px] leading-relaxed text-ink-4">
      The app never reads provider keys. The OpenCode runtime panel above lists the models this machine can use. Task sessions
      use the selected OpenCode agent model, unless <span class="font-mono text-ink-2">AGENT_BOTS_MODEL</span> is set to
      <span class="font-mono text-ink-2">provider/model</span>.
    </p>
  </Panel>

  <Panel title="Agent routing" subtitle="App agent to OpenCode agent" padded={false}>
    <div class="divide-y divide-line-soft">
      {#each snapshot.agents as agent (agent.id)}
        <div class="flex items-center gap-3 px-3.5 py-2.5">
          <Avatar name={agent.name} agentId={agent.id} size="xs" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-[11.5px] text-ink">{agent.name}</p>
            <p class="truncate text-[9.5px] text-ink-4">{agent.role}</p>
          </div>
          <span
            class="max-w-[50%] truncate rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-2"
            >{opencodeAgentId(agent)}</span
          >
        </div>
      {/each}
    </div>
    <div class="border-t border-line-soft px-3.5 py-2.5 text-[10px] leading-relaxed text-ink-4">
      Edit role prompts in <span class="font-mono text-ink-2">prompts/agents/</span> and run
      <span class="font-mono text-ink-2">npm run sync:agents</span>. Model selection stays in OpenCode.
    </div>
  </Panel>

  <Panel title="Appearance" subtitle="Dark only in this release">
    <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
      {#each tokens as token (token.name)}
        <div class="rounded-control border border-line-soft bg-canvas p-2">
          <div class="h-8 rounded border border-line-soft" style={`background: ${token.value}`}></div>
          <p class="mt-1.5 text-[10px] text-ink-2">{token.name}</p>
          <p class="font-mono text-[9px] text-ink-4">{token.value}</p>
        </div>
      {/each}
    </div>
    <p class="mt-3 text-[10.5px] leading-relaxed text-ink-3">
      System UI typography, 6 to 10 px radii, no shadows or gradients. Status color is reserved for status. A light theme is
      intentionally out of scope until the dark system settles.
    </p>
  </Panel>

  <Panel title="About">
    <dl class="space-y-2 text-[11px]">
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Version</dt>
        <dd class="font-mono text-ink-2">0.1.0</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Stack</dt>
        <dd class="text-ink-2">Tauri 2 · Svelte 5 · TypeScript · Vite · Tailwind 4</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">Repo</dt>
        <dd class="max-w-[70%] truncate font-mono text-[10px] text-ink-2">D:\Projects\agent-bots</dd>
      </div>
      <div class="flex items-center justify-between gap-4">
        <dt class="text-ink-4">State</dt>
        <dd class="text-ink-2">{live ? "Local SQLite workforce backend" : "Interface preview · no backend"}</dd>
      </div>
    </dl>
  </Panel>
</div>
