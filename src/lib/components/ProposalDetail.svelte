<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import { app, closeSheet, decideProposal, openSheet } from "../state/app.svelte.ts";
  import { memoryById, projectById, proposalById, agentById } from "../selectors.ts";
  import { workforceAvailable } from "../workforce.ts";
  import { fullTimestamp, percent, timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let busy = $state(false);

  const live = workforceAvailable();
  let proposal = $derived(app.snapshot ? proposalById(app.snapshot, id) : undefined);
  let project = $derived(proposal && app.snapshot ? projectById(app.snapshot, proposal.projectId) : undefined);
  let agent = $derived(proposal?.agentId && app.snapshot ? agentById(app.snapshot, proposal.agentId) : undefined);
  let target = $derived(proposal?.targetEntryId && app.snapshot ? memoryById(app.snapshot, proposal.targetEntryId) : undefined);

  async function decide(status: "approved" | "rejected") {
    if (!proposal) return;
    busy = true;
    try {
      await decideProposal(proposal.id, status);
    } finally {
      busy = false;
    }
  }
</script>

{#if proposal}
  <Sheet title={proposal.title} eyebrow={`${project?.key ?? proposal.projectId} · memory proposal`} onclose={closeSheet}>
    <div class="space-y-5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge status={proposal.status} />
        <span class="rounded border border-line-soft bg-raised px-1.5 py-[1px] font-mono text-[9.5px] text-ink-3">{proposal.action}</span>
        <span class="rounded border border-line-soft bg-raised px-1.5 py-[1px] text-[9.5px] text-ink-3">{proposal.scope} scope</span>
        {#if agent}<span class="text-[10px] text-ink-4">by {agent.name}</span>{/if}
      </div>

      <p class="text-[12px] leading-relaxed text-ink-2">{proposal.content}</p>

      <Section title="Reason">
        <p class="rounded-control border border-line-soft bg-canvas p-3 text-[11.5px] leading-relaxed text-ink-2">{proposal.reason}</p>
      </Section>

      <Section title="Evidence">
        <ul class="space-y-1">
          {#each proposal.evidence as item (item)}
            <li class="rounded-control border border-line-soft bg-surface px-3 py-2 text-[10.5px] text-ink-3">{item}</li>
          {/each}
        </ul>
      </Section>

      <div class="grid grid-cols-2 gap-2">
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Confidence</p>
          <p class="mt-1 font-mono text-[13px] text-ink">{percent(proposal.confidence)}</p>
          <div class="mt-1.5 h-1 overflow-hidden rounded-full bg-canvas">
            <div class="h-full rounded-full bg-accent/70" style={`width: ${Math.round(proposal.confidence * 100)}%`}></div>
          </div>
        </div>
        <div class="rounded-control border border-line-soft bg-raised px-2.5 py-2">
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Created</p>
          <p class="mt-1 text-[11px] text-ink-2">{timeAgo(proposal.createdAt)}</p>
          {#if proposal.decidedAt}<p class="mt-0.5 text-[9.5px] text-ink-4">decided {fullTimestamp(proposal.decidedAt)}</p>{/if}
        </div>
      </div>

      {#if target}
        <Section title="Target entry">
          <button
            onclick={() => openSheet("memory", target!.id)}
            class="w-full rounded-control border border-line-soft bg-raised px-3 py-2 text-left transition-colors hover:bg-hover"
          >
            <p class="truncate text-[11.5px] text-ink">{target.title}</p>
            <p class="mt-0.5 font-mono text-[9.5px] text-ink-4">{target.id}</p>
          </button>
        </Section>
      {/if}

      {#if proposal.conflictsWith}
        <Section title="Conflict">
          <p class="rounded-control border border-warn/25 bg-warn/5 px-3 py-2 font-mono text-[10.5px] text-warn">{proposal.conflictsWith}</p>
        </Section>
      {/if}

      {#if app.workforceError}
        <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10.5px] leading-relaxed text-bad" role="alert">
          {app.workforceError}
        </p>
      {/if}

      {#if proposal.status === "pending"}
        <div class="flex flex-wrap gap-2 border-t border-line-soft pt-3">
          <button
            onclick={() => decide("approved")}
            disabled={busy || app.workforceBusy}
            class="rounded-control border border-ok/40 bg-ok/10 px-3 py-1.5 text-[11px] text-ok transition-colors hover:bg-ok/15 disabled:opacity-40"
          >
            Approve proposal
          </button>
          <button
            onclick={() => decide("rejected")}
            disabled={busy || app.workforceBusy}
            class="rounded-control border border-bad/40 bg-bad/10 px-3 py-1.5 text-[11px] text-bad transition-colors hover:bg-bad/15 disabled:opacity-40"
          >
            Reject proposal
          </button>
          {#if busy}<span class="self-center text-[10px] text-ink-4" aria-live="polite">Saving…</span>{/if}
        </div>
        <p class="text-[9.5px] leading-relaxed text-ink-4">
          {live
            ? "The decision is written to the local database before the sheet updates."
            : "Browser preview: the decision is kept for this session only. Start the desktop app to store it."}
        </p>
      {:else}
        <p class="border-t border-line-soft pt-3 text-[10px] text-ink-4">
          {live
            ? `This proposal is ${proposal.status} and the decision is stored locally.`
            : `This proposal is ${proposal.status} in this session preview only.`}
        </p>
      {/if}
    </div>
  </Sheet>
{/if}
