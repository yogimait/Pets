<script lang="ts">
  import { Search } from "@lucide/svelte";
  import Panel from "../components/Panel.svelte";
  import StatusBadge from "../components/StatusBadge.svelte";
  import { memoryFiltered, pendingProposals, projectById } from "../selectors.ts";
  import { app, openSheet, runDreamer } from "../state/app.svelte.ts";
  import { workforceAvailable } from "../workforce.ts";
  import { percent, timeAgo } from "../format.ts";
  import type { MemoryKind, Snapshot } from "../types.ts";

  let { snapshot }: { snapshot: Snapshot } = $props();

  const live = workforceAvailable();

  const kinds: { id: MemoryKind | "all"; label: string }[] = [
    { id: "all", label: "All" },
    { id: "decision", label: "Decision" },
    { id: "reference", label: "Reference" },
    { id: "doc", label: "Doc" },
    { id: "result", label: "Result" },
    { id: "convention", label: "Convention" },
  ];

  let kind = $state<MemoryKind | "all">("all");
  let projectId = $state("all");
  let query = $state("");
  let dreamerProject = $state("core");
  let dreamerBusy = $state(false);

  let entries = $derived(
    memoryFiltered(snapshot, {
      kind,
      projectId: projectId === "all" ? null : projectId,
      query,
    }),
  );

  let coverage = $derived(new Set(snapshot.memory.map((entry) => entry.projectId)).size);
  let averageConfidence = $derived(
    snapshot.memory.reduce((total, entry) => total + entry.confidence, 0) / Math.max(snapshot.memory.length, 1),
  );
  let lastWrite = $derived(snapshot.memory.length > 0 ? Math.max(...snapshot.memory.map((entry) => entry.updatedAt)) : null);
  let pending = $derived(pendingProposals(snapshot));
  let proposals = $derived(
    [...snapshot.proposals].sort((a, b) => {
      if (a.status === "pending" && b.status !== "pending") return -1;
      if (a.status !== "pending" && b.status === "pending") return 1;
      return b.createdAt - a.createdAt;
    }),
  );
  let runs = $derived([...snapshot.dreamerRuns].sort((a, b) => b.startedAt - a.startedAt));
</script>

<div class="space-y-5">
  <div>
    <h1 class="text-[18px] font-medium tracking-[-0.01em] text-ink">Memory</h1>
    <p class="mt-1 text-[11.5px] text-ink-3">
      Dreamer keeps decisions, references, and results that outlive a single task.
    </p>
  </div>

  <Panel title="Dreamer" subtitle="Memory agent · on demand">
    {#snippet action()}
      {#if live}
        <div class="flex items-center gap-1.5">
          <select
            bind:value={dreamerProject}
            aria-label="Dreamer project"
            class="h-6 rounded border border-line-soft bg-surface px-1 text-[10px] text-ink-2 focus:border-accent/40 focus:outline-none"
          >
            {#each snapshot.projects as project (project.id)}<option value={project.id}>{project.key}</option>{/each}
          </select>
          <button
            onclick={async () => {
              dreamerBusy = true;
              try {
                await runDreamer(dreamerProject);
              } finally {
                dreamerBusy = false;
              }
            }}
            disabled={dreamerBusy || app.workforceBusy}
            class="rounded border border-line-soft px-1.5 py-0.5 text-[10px] text-ink-3 transition-colors hover:text-ink disabled:opacity-40"
          >
            {dreamerBusy || app.workforceBusy ? "Running" : "Run Dreamer"}
          </button>
        </div>
      {:else}
        <StatusBadge status="idle" label="Idle" />
      {/if}
    {/snippet}
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
      <div>
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Entries</p>
        <p class="mt-1 font-mono text-[15px] text-ink">{snapshot.memory.length}</p>
      </div>
      <div>
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Projects</p>
        <p class="mt-1 font-mono text-[15px] text-ink">{coverage}</p>
      </div>
      <div>
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Avg confidence</p>
        <p class="mt-1 font-mono text-[15px] text-ink">{percent(averageConfidence)}</p>
      </div>
      <div>
        <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Last write</p>
        <p class="mt-1 text-[12px] text-ink">{lastWrite ? timeAgo(lastWrite) : "none"}</p>
      </div>
    </div>
    <p class="mt-3 border-t border-line-soft pt-2.5 text-[10.5px] leading-relaxed text-ink-4">
      {live
        ? "Dreamer reads existing task and memory evidence and proposes entries. Nothing reaches memory until you approve a proposal."
        : "Entries are read-only in this browser preview. Proposals can be approved or rejected for the session only; no memory write exists without the desktop backend."}
    </p>
    {#if app.workforceError}
      <p class="mt-2 rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10.5px] leading-relaxed text-bad" role="alert">
        {app.workforceError}
      </p>
    {/if}
  </Panel>

  <div class="grid items-start gap-4 xl:grid-cols-2">
    <Panel title="Memory proposals" subtitle={`${pending.length} pending`} padded={false}>
      {#each proposals.slice(0, 5) as proposal (proposal.id)}
        {@const project = projectById(snapshot, proposal.projectId)}
        <button
          onclick={() => openSheet("proposal", proposal.id)}
          class="flex w-full items-start gap-2.5 border-b border-line-soft px-3.5 py-2.5 text-left transition-colors last:border-b-0 hover:bg-raised/50"
        >
          <StatusBadge status={proposal.status} />
          <div class="min-w-0 flex-1">
            <p class="truncate text-[11.5px] text-ink">{proposal.title}</p>
            <p class="mt-0.5 truncate text-[9.5px] text-ink-4">
              {proposal.action} · {proposal.scope} · {project?.key ?? proposal.projectId} · {percent(proposal.confidence)}
            </p>
          </div>
          <span class="shrink-0 text-[9.5px] text-ink-4">{timeAgo(proposal.createdAt)}</span>
        </button>
      {/each}
      {#if proposals.length === 0}
        <p class="px-3.5 py-4 text-[11px] text-ink-4">No memory proposals yet.</p>
      {/if}
    </Panel>

    <Panel title="Dreamer runs" subtitle="Latest consolidation passes" padded={false}>
      {#each runs as run (run.id)}
        <div class="border-b border-line-soft px-3.5 py-2.5 last:border-b-0">
          <div class="flex items-center gap-2">
            <StatusBadge status={run.status} />
            <span class="rounded border border-line-soft bg-raised px-1 py-[1px] font-mono text-[9px] text-ink-4">{run.trigger}</span>
            <span class="ml-auto text-[9.5px] text-ink-4">{timeAgo(run.startedAt)}</span>
          </div>
          <p class="mt-1.5 truncate text-[10.5px] text-ink-3">
            {run.sourcesRead.length} sources · {run.proposalIds.length} proposals · {run.parentUpdate === "sent" ? "parent notified" : "quiet"}
          </p>
        </div>
      {/each}
    </Panel>
  </div>

  <div class="space-y-2">
    <div class="flex flex-wrap items-center gap-2">
      <div class="relative min-w-[180px] flex-1 md:max-w-[280px]">
        <Search size={13} strokeWidth={1.7} class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-4" />
        <input
          bind:value={query}
          placeholder="Search memory"
          aria-label="Search memory"
          class="h-7 w-full rounded-control border border-line-soft bg-surface pl-7 pr-2 text-[11.5px] text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none"
        />
      </div>
      <select
        bind:value={projectId}
        aria-label="Filter by project"
        class="h-7 rounded-control border border-line-soft bg-surface px-1.5 text-[11px] text-ink-2 focus:border-accent/40 focus:outline-none"
      >
        <option value="all">All projects</option>
        {#each snapshot.projects as project (project.id)}<option value={project.id}>{project.key} · {project.name}</option>{/each}
      </select>
      <span class="ml-auto font-mono text-[10px] text-ink-4">{entries.length} / {snapshot.memory.length}</span>
    </div>

    <div class="flex flex-wrap items-center gap-1.5">
      {#each kinds as item (item.id)}
        <button
          onclick={() => (kind = item.id)}
          class="rounded-control border px-2 py-1 text-[10.5px] transition-colors {kind === item.id
            ? 'border-accent/30 bg-accent-soft text-ink'
            : 'border-line-soft bg-surface text-ink-3 hover:bg-hover hover:text-ink'}"
        >
          {item.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="space-y-1">
    {#each entries as entry (entry.id)}
      {@const project = projectById(snapshot, entry.projectId)}
      <button
        onclick={() => openSheet("memory", entry.id)}
        class="w-full rounded-panel border border-line-soft bg-surface px-3.5 py-2.5 text-left transition-colors hover:border-line hover:bg-raised/40"
      >
        <div class="flex items-center gap-2">
          <span class="shrink-0 rounded border border-line-soft px-1 py-[1px] font-mono text-[9px] text-ink-4">{entry.kind}</span>
          <span class="min-w-0 flex-1 truncate text-[12px] text-ink">{entry.title}</span>
          <span class="shrink-0 font-mono text-[9px] text-ink-4">{percent(entry.confidence)}</span>
          <span class="shrink-0 text-[9.5px] text-ink-4">{timeAgo(entry.updatedAt)}</span>
        </div>
        <p class="mt-1 line-clamp-1 text-[10.5px] leading-relaxed text-ink-3">{entry.content}</p>
        <div class="mt-1.5 flex flex-wrap items-center gap-2 text-[9px] text-ink-4">
          <span class="text-ink-3">{project?.key ?? entry.projectId}</span>
          {#each entry.tags as tag (tag)}<span>{tag}</span>{/each}
        </div>
      </button>
    {/each}

    {#if entries.length === 0}
      <div class="rounded-panel border border-line-soft bg-surface px-4 py-10 text-center text-[11.5px] text-ink-3">
        No memory entries match the current filters.
      </div>
    {/if}
  </div>
</div>
