<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { Direction, GestroConfig, Shortcut } from "./types";
  import GestureWheel from "./GestureWheel.svelte";
  import ShortcutCapture from "./ShortcutCapture.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import StatusBar from "./StatusBar.svelte";

  let config = $state<GestroConfig>({
    threshold: 50,
    bindings: {},
    launch_at_login: false,
  });
  let status = $state<"running" | "error">("running");
  let errorMessage = $state("");
  let editingDirection = $state<Direction | null>(null);

  async function loadConfig() {
    try {
      config = await invoke<GestroConfig>("get_config");
    } catch (e) {
      status = "error";
      errorMessage = String(e);
    }
  }

  async function saveConfig(newConfig: GestroConfig) {
    try {
      await invoke("save_config", { config: newConfig });
      config = newConfig;
    } catch (e) {
      status = "error";
      errorMessage = String(e);
    }
  }

  function handleSelectDirection(dir: Direction) {
    editingDirection = dir;
  }

  function handleSaveShortcut(shortcut: Shortcut) {
    if (!editingDirection) return;
    const newBindings = { ...config.bindings, [editingDirection]: shortcut };
    saveConfig({ ...config, bindings: newBindings });
    editingDirection = null;
  }

  function handleClearShortcut() {
    if (!editingDirection) return;
    const newBindings = { ...config.bindings };
    delete newBindings[editingDirection];
    saveConfig({ ...config, bindings: newBindings });
    editingDirection = null;
  }

  function handleCancel() {
    editingDirection = null;
  }

  function handleSettingsUpdate(newConfig: GestroConfig) {
    saveConfig(newConfig);
  }

  // Load config on mount and listen for grab errors
  $effect(() => {
    loadConfig();
    const unlisten = listen<string>("grab-error", (event) => {
      status = "error";
      errorMessage = event.payload;
    });
    return () => { unlisten.then((fn) => fn()); };
  });
</script>

<div class="app">
  <header>
    <h1>Gestro</h1>
    <p class="subtitle">Mouse Gesture Launcher</p>
  </header>

  <GestureWheel bindings={config.bindings} onSelect={handleSelectDirection} />

  <SettingsPanel {config} onUpdate={handleSettingsUpdate} />

  <footer>
    <StatusBar {status} message={errorMessage} />
  </footer>
</div>

{#if editingDirection}
  <ShortcutCapture
    direction={editingDirection}
    existing={config.bindings[editingDirection] ?? null}
    onSave={handleSaveShortcut}
    onCancel={handleCancel}
    onClear={handleClearShortcut}
  />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 24px;
    max-width: 520px;
    margin: 0 auto;
  }
  header {
    text-align: center;
  }
  h1 {
    font-size: 24px;
    font-weight: 700;
    color: var(--accent);
  }
  .subtitle {
    font-size: 13px;
    color: var(--text-muted);
    margin-top: 4px;
  }
  footer {
    margin-top: auto;
  }
</style>
