<script lang="ts">
  import type { GestroConfig } from "./types";

  interface Props {
    config: GestroConfig;
    onUpdate: (config: GestroConfig) => void;
  }
  let { config, onUpdate }: Props = $props();

  function updateThreshold(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    onUpdate({ ...config, threshold: value });
  }

  function toggleLaunchAtLogin() {
    onUpdate({ ...config, launch_at_login: !config.launch_at_login });
  }
</script>

<div class="panel">
  <h3>Settings</h3>

  <label class="setting-row">
    <div class="setting-info">
      <span class="setting-name">Gesture Threshold</span>
      <span class="setting-desc">Minimum distance (px) before a gesture is recognized</span>
    </div>
    <div class="slider-row">
      <input type="range" min="20" max="200" step="5" value={config.threshold} oninput={updateThreshold} />
      <span class="value">{config.threshold}px</span>
    </div>
  </label>

  <label class="setting-row">
    <div class="setting-info">
      <span class="setting-name">Launch at Login</span>
      <span class="setting-desc">Start Gestro automatically when you log in</span>
    </div>
    <button
      class="toggle"
      class:active={config.launch_at_login}
      onclick={toggleLaunchAtLogin}
      role="switch"
      aria-checked={config.launch_at_login}
    >
      <span class="toggle-knob"></span>
    </button>
  </label>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h3 {
    font-size: 16px;
    color: var(--text-muted);
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
    cursor: default;
  }
  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-name {
    font-size: 14px;
    font-weight: 500;
  }
  .setting-desc {
    font-size: 12px;
    color: var(--text-muted);
  }
  .slider-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .value {
    font-size: 13px;
    font-weight: 600;
    min-width: 48px;
    text-align: right;
  }
  .toggle {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 12px;
    background: var(--surface);
    padding: 2px;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle.active {
    background: var(--accent);
  }
  .toggle-knob {
    display: block;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s;
  }
  .toggle.active .toggle-knob {
    transform: translateX(20px);
  }
</style>
