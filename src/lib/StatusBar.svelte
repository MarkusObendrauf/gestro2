<script lang="ts">
  interface Props {
    status: "running" | "error";
    message?: string;
  }
  let { status, message = "" }: Props = $props();
</script>

<div class="status-bar">
  <span class="dot" class:running={status === "running"} class:error={status === "error"}></span>
  <span class="label">
    {#if status === "running"}
      Gestro is running
    {:else}
      {message || "Error"}
    {/if}
  </span>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--bg-secondary);
    border-radius: var(--radius);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .dot.running {
    background: #4caf50;
  }
  .dot.error {
    background: var(--accent);
  }
  .label {
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
