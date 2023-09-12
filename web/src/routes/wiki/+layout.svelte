<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import { fly } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'
  import WikiSidebar from '$lib/blocks/WikiSidebar.svelte'
    import type { WikiEntry } from '$lib/models/wiki'

  let toggleSidebar = false
  let screenWidth: number
  let wikiEntries: WikiEntry[] = []
  let activeId: number

  $: showSidebar = screenWidth > 1024 // lg
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row">
  {#if showSidebar}
    <div class="fixed w-1/4 h-[calc(100vh_-_4rem)] min-w-[400px] max-w-[512px] bg-base-100/40 backdrop-blur border-r border-r-base-content/5 shadow">
      <WikiSidebar bind:wiki={wikiEntries} bind:activeId />
    </div>
    <div class="w-1/4"></div>
  {:else}
    <RxButton
      size="lg"
      class="fixed right-6 bottom-6"
      on:click={() => {
        toggleSidebar = !toggleSidebar
      }}
    >
      <span class="icon-[fluent--navigation-16-regular] w-5 h-5" />
    </RxButton>
  {/if}
  <div class="flex-1">
    <slot />
  </div>
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed w-full max-w-[400px] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/5 shadow"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <WikiSidebar bind:wiki={wikiEntries} bind:activeId />
    </div>
    <RxButton
      size="lg"
      class="fixed right-6 bottom-6"
      on:click={() => {
        toggleSidebar = !toggleSidebar
      }}
    >
      <span class="icon-[fluent--dismiss-16-regular] w-5 h-5" />
    </RxButton>
  {/if}
</div>
