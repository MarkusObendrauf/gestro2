<script lang="ts">
  import type { Direction, Shortcut } from "./types";
  import { DIRECTION_LABELS, formatShortcut } from "./types";

  interface Props {
    direction: Direction;
    shortcut?: Shortcut | null;
    onClick: () => void;
  }
  let { direction, shortcut = null, onClick }: Props = $props();
</script>

<button class="slot" class:bound={shortcut !== null} onclick={onClick}>
  <span class="direction">{DIRECTION_LABELS[direction]}</span>
  {#if shortcut}
    <span class="binding">{formatShortcut(shortcut)}</span>
    {#if shortcut.label}
      <span class="label">{shortcut.label}</span>
    {/if}
  {:else}
    <span class="empty">Click to bind</span>
  {/if}
</button>

<style>
  .slot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 8px 4px;
    background: transparent;
    border-radius: var(--radius);
    transition: background 0.15s;
    width: 100%;
  }
  .slot:hover {
    background: var(--surface);
    opacity: 1;
  }
  .direction {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  .binding {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
  }
  .label {
    font-size: 11px;
    color: var(--text-muted);
  }
  .empty {
    font-size: 12px;
    color: var(--text-muted);
    opacity: 0.5;
  }
  .bound .direction {
    color: var(--accent);
  }
</style>
