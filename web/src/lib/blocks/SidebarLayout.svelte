<script lang="ts">
  import type { ComponentType } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { fly } from 'svelte/transition'

  let screenWidth: number
  let toggleLeftSidebar = false
  let toggleRightSidebar = false
  $: showLeftSidebar = screenWidth > 1024 // lg
  $: showRightSidebar = screenWidth > 1280 // lg

  export let leftSidebar: ComponentType | undefined = undefined
  export let leftProps: Record<string, unknown> | undefined = undefined
  export let rightSidebar: ComponentType | undefined = undefined
  export let rightProps: Record<string, unknown> | undefined = undefined
</script>

<svelte:window bind:innerWidth={screenWidth} />
<div class="flex-1 flex flex-row">
  {#if leftSidebar && showLeftSidebar}
    <div
      class="fixed w-1/5 top-16 left-0 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-neutral/30 backdrop-blur border-r border-r-base-content/10 print:hidden"
    >
      <svelte:component this={leftSidebar} {...leftProps} />
    </div>
    <div class="w-1/5 min-w-[24rem] max-w-[32rem] flex-shrink-0 print:hidden" />
  {:else if leftSidebar}
    <label
      class="btn no-animation bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleLeftSidebar = !toggleLeftSidebar
        }}
      />
      <span class="swap-off icon-[fluent--chevron-double-left-20-regular] fill-current w-5 h-5"></span>
      <span class="swap-on icon-[fluent--dismiss-20-regular] fill-current w-5 h-5"></span>
    </label>
  {/if}
  <slot />
  {#if rightSidebar && showRightSidebar}
    <div
      class="fixed right-0 top-16 w-1/5 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-neutral/30 backdrop-blur border-l border-l-base-content/10 print:hidden"
    >
      <svelte:component this={rightSidebar} {...rightProps} />
    </div>
    <div class="w-1/5 min-w-[24rem] max-w-[32rem] flex-shrink-0 print:hidden" />
  {:else if rightSidebar}
    <label
      class="btn no-animation bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleRightSidebar = !toggleRightSidebar
        }}
      />
      <span class="swap-off icon-[fluent--chevron-double-right-20-regular] fill-current w-5 h-5"></span>
      <span class="swap-on icon-[fluent--dismiss-20-regular] fill-current w-5 h-5"></span>
    </label>
  {/if}

  {#if leftSidebar && toggleLeftSidebar && !showLeftSidebar}
    <div
      class="fixed top-16 left-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10 print:hidden"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <svelte:component this={leftSidebar} {...leftProps} />
    </div>
  {/if}

  {#if rightSidebar && toggleRightSidebar && !showRightSidebar}
    <div
      class="fixed top-16 right-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-l border-l-base-content/10 print:hidden"
      transition:fly={{ delay: 100, duration: 300, x: 256, y: 0, opacity: 0, easing: quintOut }}
    >
      <svelte:component this={rightSidebar} {...rightProps} />
    </div>
  {/if}
</div>
