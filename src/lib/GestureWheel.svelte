<script lang="ts">
  import type { Direction, Shortcut } from "./types";
  import { ALL_DIRECTIONS, DIRECTION_LABELS, formatShortcut } from "./types";

  interface Props {
    bindings: Record<string, Shortcut>;
    onSelect: (direction: Direction) => void;
  }
  let { bindings, onSelect }: Props = $props();

  const SIZE = 280;
  const CENTER = SIZE / 2;
  const OUTER_R = 120;
  const INNER_R = 40;

  // Each direction's wedge: 45° wide, starting from the top (N) going clockwise
  // SVG angles: 0° = right (East), so we offset
  const DIRECTION_ANGLES: Record<Direction, number> = {
    N: -90,
    NE: -45,
    E: 0,
    SE: 45,
    S: 90,
    SW: 135,
    W: 180,
    NW: -135,
  };

  function wedgePath(dir: Direction): string {
    const angle = DIRECTION_ANGLES[dir];
    const halfSector = 22.5;
    const startAngle = ((angle - halfSector) * Math.PI) / 180;
    const endAngle = ((angle + halfSector) * Math.PI) / 180;

    const x1o = CENTER + OUTER_R * Math.cos(startAngle);
    const y1o = CENTER + OUTER_R * Math.sin(startAngle);
    const x2o = CENTER + OUTER_R * Math.cos(endAngle);
    const y2o = CENTER + OUTER_R * Math.sin(endAngle);
    const x1i = CENTER + INNER_R * Math.cos(endAngle);
    const y1i = CENTER + INNER_R * Math.sin(endAngle);
    const x2i = CENTER + INNER_R * Math.cos(startAngle);
    const y2i = CENTER + INNER_R * Math.sin(startAngle);

    return [
      `M ${x1o} ${y1o}`,
      `A ${OUTER_R} ${OUTER_R} 0 0 1 ${x2o} ${y2o}`,
      `L ${x1i} ${y1i}`,
      `A ${INNER_R} ${INNER_R} 0 0 0 ${x2i} ${y2i}`,
      "Z",
    ].join(" ");
  }

  function labelPos(dir: Direction): { x: number; y: number } {
    const angle = (DIRECTION_ANGLES[dir] * Math.PI) / 180;
    const r = (OUTER_R + INNER_R) / 2;
    return {
      x: CENTER + r * Math.cos(angle),
      y: CENTER + r * Math.sin(angle),
    };
  }
</script>

<div class="wheel-container">
  <svg width={SIZE} height={SIZE} viewBox="0 0 {SIZE} {SIZE}">
    {#each ALL_DIRECTIONS as dir}
      {@const pos = labelPos(dir)}
      {@const bound = bindings[dir]}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <g
        class="wedge"
        class:bound={!!bound}
        role="button"
        tabindex="0"
        onclick={() => onSelect(dir)}
      >
        <path d={wedgePath(dir)} />
        <text x={pos.x} y={pos.y - 6} text-anchor="middle" dominant-baseline="middle" class="dir-label">
          {DIRECTION_LABELS[dir]}
        </text>
        {#if bound}
          <text x={pos.x} y={pos.y + 8} text-anchor="middle" dominant-baseline="middle" class="shortcut-label">
            {bound.label || formatShortcut(bound)}
          </text>
        {/if}
      </g>
    {/each}
    <!-- Center circle -->
    <circle cx={CENTER} cy={CENTER} r={INNER_R - 2} class="center" />
    <text x={CENTER} y={CENTER} text-anchor="middle" dominant-baseline="middle" class="center-text">
      G
    </text>
  </svg>
</div>

<style>
  .wheel-container {
    display: flex;
    justify-content: center;
    padding: 16px 0;
  }
  .wedge path {
    fill: var(--surface);
    stroke: var(--bg);
    stroke-width: 2;
    cursor: pointer;
    transition: fill 0.15s;
  }
  .wedge:hover path {
    fill: #1a4a7a;
  }
  .wedge.bound path {
    fill: #1a3a5c;
  }
  .wedge.bound:hover path {
    fill: #1a4a7a;
  }
  .dir-label {
    font-size: 9px;
    fill: var(--text-muted);
    pointer-events: none;
    font-weight: 600;
    text-transform: uppercase;
  }
  .shortcut-label {
    font-size: 7px;
    fill: var(--accent);
    pointer-events: none;
  }
  .center {
    fill: var(--bg);
    stroke: var(--surface);
    stroke-width: 2;
  }
  .center-text {
    font-size: 20px;
    font-weight: 700;
    fill: var(--accent);
  }
</style>
