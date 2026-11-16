<script lang="ts">
  import Sheet from "./Sheet.svelte";
  import { app, closeSheet, createProject, openSheet } from "../state/app.svelte.ts";
  import { workforceAvailable, workforceInspectProject, type ProjectInspection } from "../workforce.ts";

  const live = workforceAvailable();
  const inputClass =
    "h-8 w-full rounded-control border border-line-soft bg-canvas px-2.5 text-[11.5px] text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none";

  let name = $state("");
  let key = $state("");
  let keyTouched = $state(false);
  let repoPath = $state("");
  let pathTouched = $state(false);
  let description = $state("");
  let context = $state("");
  let instructions = $state("");
  let createFolder = $state(false);
  let createPolicy = $state(true);
  let createReadme = $state(true);
  let initGit = $state(false);
  let autonomy = $state(true);
  let maxPhases = $state(6);
  let inspection = $state<ProjectInspection | null>(null);
  let inspecting = $state(false);
  let busy = $state(false);
  let error = $state("");

  $effect(() => {
    const value = name.trim();
    if (pathTouched || !value) return;
    const slug = value
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
    repoPath = `D:\\Projects\\${slug || "new-project"}`;
    createFolder = true;
  });

  $effect(() => {
    const value = name.trim();
    if (keyTouched || !value) return;
    const words = value.split(/\s+/).filter(Boolean);
    const candidate =
      words.length > 1
        ? words.map((word) => word[0]).join("")
        : value.replace(/[^a-z0-9]/gi, "").slice(0, 5);
    const suggested = candidate.replace(/[^a-z0-9]/gi, "").toUpperCase().slice(0, 8);
    if (suggested.length >= 2) key = suggested;
  });

  let canSubmit = $derived(
    live &&
      !busy &&
      name.trim().length > 0 &&
      /^[A-Za-z0-9]{2,8}$/.test(key.trim()) &&
      repoPath.trim().length > 0,
  );

  async function inspect() {
    if (!repoPath.trim()) {
      inspection = null;
      return;
    }
    inspecting = true;
    error = "";
    try {
      inspection = await workforceInspectProject(repoPath.trim());
    } catch (cause) {
      inspection = null;
      error = String(cause);
    } finally {
      inspecting = false;
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!canSubmit) return;
    busy = true;
    error = "";
    try {
      const id = await createProject({
        name: name.trim(),
        key: key.trim().toUpperCase(),
        repoPath: repoPath.trim(),
        description: description.trim(),
        context: context.trim(),
        instructions: instructions.trim(),
        createFolder,
        createPolicy,
        createReadme,
        initGit,
        autonomy,
        maxPhases,
      });
      if (id) {
        closeSheet();
        openSheet("project", id);
      } else if (app.workforceError) {
        error = app.workforceError;
      }
    } finally {
      busy = false;
    }
  }
</script>

<Sheet title="New or import project" eyebrow="Project onboarding" onclose={closeSheet}>
  <div class="space-y-5">
    {#if !live}
      <p class="rounded-control border border-warn/25 bg-warn/5 px-3 py-2 text-[10.5px] leading-relaxed text-warn">
        Project creation needs the desktop app. The browser preview keeps the fixture projects only.
      </p>
    {/if}

    <p class="text-[11px] leading-relaxed text-ink-3">
      Add an existing folder, or create a new one. The folder becomes the working directory for every session in this
      project. A project policy is required before real work starts.
    </p>

    <form class="space-y-4" onsubmit={submit}>
      <div class="grid grid-cols-[minmax(0,1fr)_112px] gap-2">
        <label class="space-y-1">
          <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Project name</span>
          <input bind:value={name} class={inputClass} placeholder="My app" autocomplete="off" />
        </label>
        <label class="space-y-1">
          <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Key</span>
          <input
            bind:value={key}
            oninput={() => (keyTouched = true)}
            maxlength={8}
            class={inputClass}
            placeholder="MYAPP"
            autocomplete="off"
            aria-describedby="project-key-help"
          />
        </label>
      </div>
      <p id="project-key-help" class="-mt-2 text-[9.5px] text-ink-4">2 to 8 letters or digits. Used in task references.</p>

      <label class="space-y-1">
        <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Folder path</span>
        <div class="flex gap-2">
          <input
            bind:value={repoPath}
            oninput={() => {
              pathTouched = true;
              inspection = null;
            }}
            class={inputClass}
            placeholder="D:\Projects\my-app"
            autocomplete="off"
            spellcheck={false}
          />
          <button
            type="button"
            onclick={inspect}
            disabled={!live || inspecting || !repoPath.trim()}
            class="h-8 shrink-0 rounded-control border border-line-soft bg-surface px-2.5 text-[10.5px] text-ink-2 transition-colors hover:bg-hover hover:text-ink disabled:opacity-40"
          >
            {inspecting ? "Checking" : "Check"}
          </button>
        </div>
      </label>

      {#if inspection}
        <div class="rounded-control border border-line-soft bg-raised px-3 py-2.5 text-[10.5px] leading-relaxed text-ink-3">
          <p class="text-ink-2">
            {inspection.exists
              ? inspection.isDirectory
                ? "Folder found"
                : "Path is not a folder"
              : "Folder does not exist"}
          </p>
          {#if inspection.exists && inspection.isDirectory}
            <p class="mt-1">
              {inspection.entryCount} entries · AGENTS.md {inspection.hasAgents ? "found" : "missing"} · README.md
              {inspection.hasReadme ? "found" : "missing"} · Git {inspection.isGit ? "initialized" : "not initialized"}
            </p>
            {#if !inspection.hasAgents}
              <p class="mt-1 text-warn">A real run is blocked until AGENTS.md exists.</p>
            {/if}
          {:else if !inspection.exists}
            <p class="mt-1">Enable “Create folder” below, or choose an existing path.</p>
          {/if}
        </div>
      {/if}

      <label class="space-y-1">
        <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Description</span>
        <input bind:value={description} class={inputClass} placeholder="What this project is" autocomplete="off" />
      </label>

      <label class="space-y-1">
        <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Project context</span>
        <textarea
          bind:value={context}
          rows={2}
          class="w-full resize-y rounded-control border border-line-soft bg-canvas p-2.5 text-[11.5px] leading-relaxed text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none"
          placeholder="Current state, stack, constraints, what already exists"
        ></textarea>
      </label>

      <label class="space-y-1">
        <span class="block text-[10px] uppercase tracking-[0.1em] text-ink-4">Project instructions</span>
        <textarea
          bind:value={instructions}
          rows={2}
          class="w-full resize-y rounded-control border border-line-soft bg-canvas p-2.5 text-[11.5px] leading-relaxed text-ink placeholder:text-ink-4 focus:border-accent/40 focus:outline-none"
          placeholder="How agents should work in this project"
        ></textarea>
      </label>

      <div class="space-y-2 rounded-control border border-line-soft bg-surface p-3">
        <label class="flex items-start gap-2 text-[11px] text-ink-2">
          <input type="checkbox" bind:checked={autonomy} class="mt-0.5 size-3 accent-[#8ca6c0]" />
          <span>
            <span class="text-ink">Run autonomously after each phase</span>
            <span class="mt-0.5 block text-[9.5px] text-ink-4">Re-plan remaining work when all current tasks are done, up to the phase limit.</span>
          </span>
        </label>
        <label class="flex items-center gap-2 text-[11px] text-ink-2">
          <span class="text-ink-4">Phase limit</span>
          <input
            type="number"
            bind:value={maxPhases}
            min="1"
            max="20"
            class="h-7 w-16 rounded-control border border-line-soft bg-canvas px-2 text-[11px] text-ink-2"
          />
        </label>

        <label class="flex items-start gap-2 text-[11px] text-ink-2">
          <input type="checkbox" bind:checked={createFolder} class="mt-0.5 size-3 accent-[#8ca6c0]" />
          <span>
            <span class="text-ink">Create the folder if it does not exist</span>
            <span class="mt-0.5 block text-[9.5px] text-ink-4">Required for the “idea only” case.</span>
          </span>
        </label>
        <label class="flex items-start gap-2 text-[11px] text-ink-2">
          <input type="checkbox" bind:checked={createPolicy} class="mt-0.5 size-3 accent-[#8ca6c0]" />
          <span>
            <span class="text-ink">Create a starter AGENTS.md if missing</span>
            <span class="mt-0.5 block text-[9.5px] text-ink-4">Minimal project policy. Global policy still applies. Existing files are never overwritten.</span>
          </span>
        </label>
        <label class="flex items-start gap-2 text-[11px] text-ink-2">
          <input type="checkbox" bind:checked={createReadme} class="mt-0.5 size-3 accent-[#8ca6c0]" />
          <span>
            <span class="text-ink">Create a starter README.md if missing</span>
            <span class="mt-0.5 block text-[9.5px] text-ink-4">Setup and checks need a place to start.</span>
          </span>
        </label>
        <label class="flex items-start gap-2 text-[11px] text-ink-2">
          <input type="checkbox" bind:checked={initGit} class="mt-0.5 size-3 accent-[#8ca6c0]" />
          <span>
            <span class="text-ink">Initialize a Git repository</span>
            <span class="mt-0.5 block text-[9.5px] text-ink-4">Runs `git init` only. No commit is created.</span>
          </span>
        </label>
      </div>

      {#if error || app.workforceError}
        <p class="rounded-control border border-bad/30 bg-bad/5 px-2.5 py-2 text-[10.5px] leading-relaxed text-bad" role="alert">
          {error || app.workforceError}
        </p>
      {/if}

      <div class="flex items-center justify-end gap-2 border-t border-line-soft pt-3">
        <button
          type="button"
          onclick={closeSheet}
          class="rounded-control border border-line-soft px-3 py-1.5 text-[11px] text-ink-3 transition-colors hover:text-ink"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={!canSubmit}
          class="rounded-control border border-accent/40 bg-accent-soft px-3 py-1.5 text-[11px] text-ink transition-colors hover:border-accent/60 disabled:opacity-40"
        >
          {busy ? "Saving" : "Save project"}
        </button>
      </div>
    </form>
  </div>
</Sheet>
