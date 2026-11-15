<script lang="ts">
  import { Check } from "@lucide/svelte";
  import Sheet from "./Sheet.svelte";
  import Section from "./Section.svelte";
  import Avatar from "./Avatar.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import { abortTask, acceptTask, app, closeSheet, openSheet, openTask, replyTaskPermission, runTask } from "../state/app.svelte.ts";
  import { agentById, projectById, taskById, taskStepProgress } from "../selectors.ts";
  import { workforceAvailable } from "../workforce.ts";
  import { fullTimestamp, timeAgo } from "../format.ts";

  let { id }: { id: string } = $props();

  let task = $derived(app.snapshot ? taskById(app.snapshot, id) : undefined);
  let project = $derived(task && app.snapshot ? projectById(app.snapshot, task.projectId) : undefined);
  let owner = $derived(task && app.snapshot ? agentById(app.snapshot, task.ownerId) : undefined);
  let progress = $derived(task ? taskStepProgress(task) : 0);
  const live = workforceAvailable();

  const priorityTone: Record<string, string> = {
    low: "text-ink-4",
    normal: "text-ink-3",
    high: "text-warn",
    critical: "text-bad",
  };

  function senderName(senderId: string): string {
    if (!app.snapshot) return senderId;
    return agentById(app.snapshot, senderId)?.name ?? senderId;
  }
</script>

{#if task}
  <Sheet title={task.title} eyebrow={`${project?.key ?? task.projectId} · ${task.id}`} onclose={closeSheet}>
    <div class="space-y-5">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge status={task.status} />
        <span class="rounded border border-line-soft bg-raised px-1.5 py-[1px] text-[10.5px] {priorityTone[task.priority]}"
          >{task.priority} priority</span
        >
        <span class="text-[10px] text-ink-4">updated {timeAgo(task.updatedAt)}</span>
      </div>

      {#if live && ["queued", "blocked", "running", "review"].includes(task.status)}
        <div class="flex flex-wrap items-center gap-2">
          {#if task.status === "queued" || task.status === "blocked"}
            <button
              onclick={() => runTask(task!.id)}
              disabled={app.workforceBusy}
              class="rounded-control border border-accent/40 bg-accent-soft px-3 py-1.5 text-[11px] text-ink transition-colors hover:border-accent/60 disabled:opacity-40"
            >
              {task.status === "blocked" ? "Retry" : "Run now"}
            </button>
          {:else if task.status === "review"}
            <button
              onclick={() => acceptTask(task!.id)}
              disabled={app.workforceBusy || !task.sessionId}
              class="rounded-control border border-ok/40 bg-ok/10 px-3 py-1.5 text-[11px] text-ok transition-colors hover:bg-ok/15 disabled:opacity-40"
            >
              Accept result
            </button>
            {#if !task.sessionId}
              <span class="text-[10px] text-ink-4">This seeded review has no linked session.</span>
            {/if}
          {:else}
            <button
              onclick={() => abortTask(task!.id)}
              disabled={app.workforceBusy}
              class="rounded-control border border-bad/40 bg-bad/10 px-3 py-1.5 text-[11px] text-bad transition-colors hover:bg-bad/15 disabled:opacity-40"
            >
              Abort
            </button>
          {/if}
          {#if app.workforceBusy}
            <span class="text-[10px] text-ink-4" aria-live="polite">Applying…</span>
          {/if}
        </div>
      {/if}

      {#if task.permission}
        <div class="rounded-control border border-warn/35 bg-warn/5 p-3" role="alert">
          <div class="flex items-center justify-between gap-2">
            <p class="text-[10px] uppercase tracking-[0.1em] text-warn">Permission needed</p>
            <span class="font-mono text-[9px] text-ink-4">{task.permission.action}</span>
          </div>
          <p class="mt-1 break-words font-mono text-[10px] text-ink-2">{(task.permission.resources ?? []).join(" · ")}</p>
          {#if task.permission.message}
            <p class="mt-1 text-[10px] leading-relaxed text-ink-3">{task.permission.message}</p>
          {/if}
          <div class="mt-2 flex flex-wrap gap-1.5">
            <button
              onclick={() => replyTaskPermission(task!.id, task!.permission!.id, "once")}
              disabled={app.workforceBusy}
              class="rounded border border-ok/40 bg-ok/10 px-2 py-0.5 text-[10px] text-ok disabled:opacity-40"
            >
              Allow once
            </button>
            <button
              onclick={() => replyTaskPermission(task!.id, task!.permission!.id, "always")}
              disabled={app.workforceBusy}
              class="rounded border border-line-soft bg-surface px-2 py-0.5 text-[10px] text-ink-2 disabled:opacity-40"
            >
              Always
            </button>
            <button
              onclick={() => replyTaskPermission(task!.id, task!.permission!.id, "reject")}
              disabled={app.workforceBusy}
              class="rounded border border-bad/40 bg-bad/10 px-2 py-0.5 text-[10px] text-bad disabled:opacity-40"
            >
              Deny
            </button>
          </div>
        </div>
      {/if}

      {#if app.workforceError}
        <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10.5px] leading-relaxed text-bad" role="alert">
          {app.workforceError}
        </p>
      {/if}

      <p class="text-[12px] leading-relaxed text-ink-2">{task.description}</p>

      <div class="grid grid-cols-2 gap-2">
        <button
          onclick={() => owner && openSheet("agent", owner.id)}
          class="rounded-control border border-line-soft bg-raised px-2.5 py-2 text-left transition-colors hover:border-line hover:bg-hover"
        >
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Owner</p>
          <div class="mt-1.5 flex items-center gap-1.5">
            <Avatar name={owner?.name ?? task.ownerId} size="xs" />
            <span class="truncate text-[11.5px] text-ink">{owner?.name ?? task.ownerId}</span>
          </div>
        </button>
        <button
          onclick={() => project && openSheet("project", project.id)}
          class="rounded-control border border-line-soft bg-raised px-2.5 py-2 text-left transition-colors hover:border-line hover:bg-hover"
        >
          <p class="text-[9.5px] uppercase tracking-[0.1em] text-ink-4">Project</p>
          <span class="mt-1.5 block truncate text-[11.5px] text-ink">{project?.name ?? task.projectId}</span>
        </button>
      </div>

      {#if task.steps.length > 0}
        <Section title={`Steps · ${task.steps.filter((step) => step.done).length}/${task.steps.length}`}>
          <div class="mb-2 h-1 overflow-hidden rounded-full bg-raised">
            <div class="h-full rounded-full bg-accent/70" style={`width: ${Math.round(progress * 100)}%`}></div>
          </div>
          <ul class="space-y-1.5">
            {#each task.steps as step (step.label)}
              <li class="flex items-start gap-2 text-[11.5px] {step.done ? 'text-ink-3' : 'text-ink-2'}">
                <span
                  class="mt-[1px] grid size-4 shrink-0 place-items-center rounded-full border {step.done
                    ? 'border-ok/40 text-ok'
                    : 'border-line text-transparent'}"
                >
                  <Check size={10} strokeWidth={2.2} />
                </span>
                <span class={step.done ? "line-through decoration-line" : ""}>{step.label}</span>
              </li>
            {/each}
          </ul>
        </Section>
      {/if}

      {#if task.acceptance}
        <Section title="Acceptance">
          <p class="whitespace-pre-wrap rounded-control border border-line-soft bg-canvas p-3 text-[11.5px] leading-relaxed text-ink-2">{task.acceptance}</p>
        </Section>
      {/if}

      {#if task.result}
        <Section title="Result">
          <p class="max-h-64 overflow-y-auto whitespace-pre-wrap rounded-control border border-line-soft bg-canvas p-3 text-[11.5px] leading-relaxed text-ink-2">{task.result}</p>
        </Section>
      {/if}

      {#if task.notes}
        <Section title="Notes">
          <p class="max-h-52 overflow-y-auto whitespace-pre-wrap rounded-control border border-line-soft bg-canvas p-3 text-[11.5px] leading-relaxed text-ink-2">{task.notes}</p>
        </Section>
      {/if}

      {#if task.dependsOn && task.dependsOn.length > 0}
        <Section title="Depends on">
          <div class="flex flex-wrap gap-1.5">
            {#each task.dependsOn as dependencyId (dependencyId)}
              {@const dependency = app.snapshot ? taskById(app.snapshot, dependencyId) : undefined}
              <button
                onclick={() => openTask(dependencyId)}
                class="max-w-full truncate rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-3 transition-colors hover:border-line hover:text-ink"
              >
                {dependency ? `${dependency.id} · ${dependency.title}` : dependencyId}
              </button>
            {/each}
          </div>
        </Section>
      {/if}

      {#if task.sessionId}
        <Section title="OpenCode session">
          <p class="truncate rounded-control border border-line-soft bg-canvas px-3 py-2 font-mono text-[10.5px] text-ink-2">
            {task.sessionId}
          </p>
        </Section>
      {/if}

      {#if task.tags.length > 0}
        <Section title="Tags">
          <div class="flex flex-wrap gap-1.5">
            {#each task.tags as tag (tag)}
              <span class="rounded border border-line-soft bg-raised px-1.5 py-0.5 font-mono text-[9.5px] text-ink-3">{tag}</span>
            {/each}
          </div>
        </Section>
      {/if}

      <Section title={`Thread · ${task.messages.length}`}>
        <div class="space-y-2.5">
          {#each task.messages as message (message.id)}
            <div class="flex gap-2">
              <Avatar name={senderName(message.senderId)} size="xs" />
              <div class="min-w-0 flex-1">
                <div class="flex items-baseline gap-2">
                  <span class="text-[10.5px] text-ink-2">{senderName(message.senderId)}</span>
                  <span class="font-mono text-[9px] text-ink-4">{timeAgo(message.at)}</span>
                </div>
                <p class="mt-0.5 text-[11.5px] leading-relaxed text-ink-2">{message.text}</p>
              </div>
            </div>
          {/each}
          {#if task.messages.length === 0}
            <p class="text-[11px] text-ink-4">No messages on this task.</p>
          {/if}
        </div>
      </Section>

      <div class="flex items-center justify-between border-t border-line-soft pt-3 text-[9.5px] text-ink-4">
        <span>created {fullTimestamp(task.createdAt)}</span>
        <span>updated {fullTimestamp(task.updatedAt)}</span>
      </div>

      <button
        onclick={() => openTask(task.id)}
        class="w-full rounded-control border border-line-soft bg-raised px-3 py-2 text-[11px] text-ink-2 transition-colors hover:bg-hover hover:text-ink"
      >
        Open in task board
      </button>
    </div>
  </Sheet>
{/if}
