<script lang="ts">
  import type { Direction, Shortcut } from "./types";
  import { DIRECTION_LABELS } from "./types";

  interface Props {
    direction: Direction;
    onSave: (shortcut: Shortcut) => void;
    onCancel: () => void;
    onClear: () => void;
    existing?: Shortcut | null;
  }
  let { direction, onSave, onCancel, onClear, existing = null }: Props = $props();

  let modifiers = $state<string[]>([]);
  let key = $state("");
  let label = $state(existing?.label ?? "");
  let listening = $state(true);

  const MODIFIER_KEYS = new Set([
    "Control",
    "Alt",
    "Shift",
    "Meta",
  ]);

  function handleKeyDown(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();

    if (!listening) return;

    if (e.key === "Escape") {
      onCancel();
      return;
    }

    // Collect modifiers
    const mods: string[] = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.altKey) mods.push("Alt");
    if (e.shiftKey) mods.push("Shift");
    if (e.metaKey) mods.push("Meta");

    // If only a modifier was pressed, update display but don't save yet
    if (MODIFIER_KEYS.has(e.key)) {
      modifiers = mods;
      return;
    }

    // Non-modifier key pressed — finalize the shortcut
    modifiers = mods;
    key = mapKey(e);
    listening = false;
  }

  function mapKey(e: KeyboardEvent): string {
    if (e.code.startsWith("Key")) return e.code.slice(3); // KeyA -> A
    if (e.code.startsWith("Digit")) return e.code.slice(5); // Digit1 -> 1
    if (e.code.startsWith("Arrow")) return e.code.slice(5); // ArrowUp -> Up

    const mapped: Record<string, string> = {
      Space: "Space",
      Enter: "Return",
      Tab: "Tab",
      Escape: "Escape",
      Backspace: "Backspace",
      Delete: "Delete",
      Home: "Home",
      End: "End",
      PageUp: "PageUp",
      PageDown: "PageDown",
    };

    // F-keys
    if (e.code.startsWith("F") && e.code.length <= 3) return e.code;

    return mapped[e.code] ?? e.key;
  }

  function save() {
    if (!key) return;
    onSave({
      modifiers,
      key,
      label: label || undefined,
    });
  }

  function displayCombo(): string {
    const parts = [...modifiers];
    if (key) parts.push(key);
    return parts.length ? parts.join(" + ") : "Press a key combo...";
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onkeydown={handleKeyDown}>
  <div class="modal">
    <h3>Bind {DIRECTION_LABELS[direction]}</h3>

    <div class="capture-area" tabindex="-1">
      <span class="combo" class:placeholder={!key && modifiers.length === 0}>
        {displayCombo()}
      </span>
    </div>

    <label class="label-row">
      <span>Label (optional)</span>
      <input type="text" bind:value={label} placeholder="e.g. Volume Up" />
    </label>

    <div class="actions">
      {#if existing}
        <button class="btn-clear" onclick={onClear}>Clear</button>
      {/if}
      <button class="btn-cancel" onclick={onCancel}>Cancel</button>
      <button class="btn-save" disabled={!key} onclick={save}>Save</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg-secondary);
    border-radius: 12px;
    padding: 24px;
    width: 340px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h3 {
    font-size: 18px;
    text-align: center;
  }
  .capture-area {
    background: var(--bg);
    border: 2px solid var(--surface);
    border-radius: var(--radius);
    padding: 20px;
    text-align: center;
    font-size: 20px;
    font-weight: 600;
    outline: none;
  }
  .capture-area:focus-within {
    border-color: var(--accent);
  }
  .placeholder {
    color: var(--text-muted);
    font-weight: 400;
  }
  .label-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    color: var(--text-muted);
  }
  .label-row input {
    background: var(--bg);
    border: 1px solid var(--surface);
    border-radius: var(--radius);
    padding: 8px;
    color: var(--text);
    font-size: 14px;
    outline: none;
  }
  .label-row input:focus {
    border-color: var(--accent);
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .btn-save {
    background: var(--accent);
    color: white;
  }
  .btn-save:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .btn-cancel {
    background: var(--surface);
    color: var(--text);
  }
  .btn-clear {
    background: transparent;
    color: var(--accent);
    margin-right: auto;
  }
</style>
