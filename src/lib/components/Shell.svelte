<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import Topbar from "./Topbar.svelte";
  import Palette from "./Palette.svelte";
  import AgentDetail from "./AgentDetail.svelte";
  import ProjectDetail from "./ProjectDetail.svelte";
  import ProjectForm from "./ProjectForm.svelte";
  import TaskDetail from "./TaskDetail.svelte";
  import MemoryDetail from "./MemoryDetail.svelte";
  import ProposalDetail from "./ProposalDetail.svelte";
  import Overview from "../views/Overview.svelte";
  import Agents from "../views/Agents.svelte";
  import Projects from "../views/Projects.svelte";
  import Tasks from "../views/Tasks.svelte";
  import ActivityView from "../views/Activity.svelte";
  import Memory from "../views/Memory.svelte";
  import Settings from "../views/Settings.svelte";
  import { app, closeSheet } from "../state/app.svelte.ts";

  let main = $state<HTMLElement | null>(null);

  $effect(() => {
    app.view;
    if (main) main.scrollTop = 0;
  });

  function onKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      app.paletteOpen = !app.paletteOpen;
      return;
    }
    if (event.key === "Escape") {
      if (app.paletteOpen) app.paletteOpen = false;
      else if (app.sheet) closeSheet();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="flex h-screen overflow-hidden bg-canvas">
  <Sidebar />

  <div class="flex min-w-0 flex-1 flex-col">
    <Topbar />
    <main bind:this={main} class="min-h-0 flex-1 overflow-y-auto">
      {#if app.snapshot}
        {@const snapshot = app.snapshot}
        <div class="mx-auto max-w-[1120px] px-5 py-5">
          {#if app.view === "overview"}
            <Overview {snapshot} />
          {:else if app.view === "agents"}
            <Agents {snapshot} />
          {:else if app.view === "projects"}
            <Projects {snapshot} />
          {:else if app.view === "tasks"}
            <Tasks {snapshot} />
          {:else if app.view === "activity"}
            <ActivityView {snapshot} />
          {:else if app.view === "memory"}
            <Memory {snapshot} />
          {:else}
            <Settings {snapshot} />
          {/if}
        </div>
      {/if}
    </main>
  </div>
</div>

{#if app.paletteOpen}
  <Palette />
{/if}

{#if app.sheet?.kind === "agent"}
  <AgentDetail id={app.sheet.id} />
{:else if app.sheet?.kind === "project"}
  <ProjectDetail id={app.sheet.id} />
{:else if app.sheet?.kind === "project-new"}
  <ProjectForm />
{:else if app.sheet?.kind === "task"}
  <TaskDetail id={app.sheet.id} />
{:else if app.sheet?.kind === "memory"}
  <MemoryDetail id={app.sheet.id} />
{:else if app.sheet?.kind === "proposal"}
  <ProposalDetail id={app.sheet.id} />
{/if}
