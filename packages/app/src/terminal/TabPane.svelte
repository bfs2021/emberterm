<script lang="ts">
  import Grid from "./Grid.svelte";
  import { SplitContainer } from "@emberterm/ui";
  import ToolCard from "../agent/ToolCard.svelte";
  import BlockList from "../agent/BlockList.svelte";
  import DiffView from "../agent/DiffView.svelte";
  import ApprovalSheet from "../agent/ApprovalSheet.svelte";
  import HelloBanner from "../chrome/HelloBanner.svelte";
  import type { Tab, Block, ToolCall } from "../stores/app.js";
  import type { AgentDiffPayload, AgentApprovalPayload } from "../ipc/events.js";

  interface Props {
    tab: Tab;
    isActive: boolean;
    blocks: Block[];
    splitBlocks: Block[];
    toolCalls: ToolCall[];
    pendingDiff?: AgentDiffPayload;
    pendingApproval?: AgentApprovalPayload;
    pendingHello?: { name: string; version: string };
    onExit: () => void;
    onSplitExit: () => void;
    onResize?: (cols: number, rows: number) => void;
    onCancelTool: (toolId: string) => void;
    onDiffAction: (id: string, result: string) => void;
    onApprovalAction: (id: string, result: string) => void;
    onDismissHello: () => void;
    onCloseSplit: () => void;
  }

  let {
    tab, isActive, blocks, splitBlocks, toolCalls,
    pendingDiff, pendingApproval, pendingHello,
    onExit, onSplitExit, onResize,
    onCancelTool, onDiffAction, onApprovalAction, onDismissHello, onCloseSplit,
  }: Props = $props();

  let splitRatio = $state(0.5);
</script>

<div class="tab-pane" class:hidden={!isActive}>
  <!-- Terminal grid(s) — possibly split -->
  {#if tab.splitSessionId}
    <SplitContainer
      direction={tab.splitDirection ?? "horizontal"}
      bind:ratio={splitRatio}
      onClose={onCloseSplit}
    >
      {#snippet primary()}
        <Grid sessionId={tab.sessionId} active={isActive} {onExit} {onResize} />
        <BlockList blocks={blocks} />
      {/snippet}
      {#snippet secondary()}
        <Grid sessionId={tab.splitSessionId!} active={isActive} onExit={onSplitExit} />
        <BlockList blocks={splitBlocks} />
      {/snippet}
    </SplitContainer>
  {:else}
    <Grid sessionId={tab.sessionId} active={isActive} {onExit} {onResize} />
    <BlockList blocks={blocks} />
  {/if}

  <!-- Agent overlays — only rendered on the active tab -->
  {#if isActive}
    <!-- Tool cards -->
    {#each toolCalls.slice(-5) as call (call.id)}
      <div class="agent-overlay agent-overlay--cards">
        <ToolCard
          id={call.id}
          name={call.name}
          summary={call.summary}
          icon={call.icon}
          status={call.status}
          progressPct={call.progressPct}
          progressMessage={call.progressMessage}
          durationMs={call.durationMs}
          args={call.args}
          onCancel={call.status === "running" ? () => onCancelTool(call.id) : undefined}
        />
      </div>
    {/each}

    <!-- Diff viewer -->
    {#if pendingDiff}
      <div class="agent-overlay agent-overlay--diff">
        <DiffView
          id={pendingDiff.id}
          path={pendingDiff.path}
          patch={pendingDiff.patch}
          actions={pendingDiff.actions}
          onAction={onDiffAction}
        />
      </div>
    {/if}

    <!-- Approval sheet -->
    {#if pendingApproval}
      <ApprovalSheet
        id={pendingApproval.id}
        title={pendingApproval.title}
        detail={pendingApproval.detail}
        risk={pendingApproval.risk}
        actions={pendingApproval.actions}
        onAction={onApprovalAction}
      />
    {/if}

    <!-- Agent hello banner -->
    {#if pendingHello}
      <HelloBanner
        name={pendingHello.name}
        version={pendingHello.version}
        onDismiss={onDismissHello}
      />
    {/if}
  {/if}
</div>

<style>
  .tab-pane {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
  }

  .tab-pane.hidden { display: none; }

  /* Shared overlay base — all agent UI floats above the terminal */
  .agent-overlay {
    position: absolute;
    left: 8px;
    right: 8px;
    z-index: 10;
    pointer-events: none;
  }

  .agent-overlay > :global(*) { pointer-events: all; }

  /* Tool cards: bottom-right stack */
  .agent-overlay--cards { bottom: 8px; }

  /* Diff viewer: above the tool card area */
  .agent-overlay--diff {
    bottom: 80px;
    max-width: 680px;
  }
</style>
