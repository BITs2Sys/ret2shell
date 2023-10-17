<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { user } from '$lib/stores/user'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import { fly } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'

  let toggleSidebar = false
  let screenWidth: number
  let error = 200
  $: showSidebar = screenWidth > 1024 // lg

  if (!$user.isLoggedIn) {
    goto(`/account/login?redirect=${$page.url.pathname}`)
  }
</script>

<svelte:head><title>{$i18n.t('account.settings')} - {$platform.name}</title></svelte:head>
<svelte:window bind:innerWidth={screenWidth} />
<div class="flex-1 flex flex-row">
  {#if showSidebar}
    <div
      class="fixed w-1/5 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-neutral/30 backdrop-blur border-r border-r-base-content/10 print:hidden"
    >
      <Sidebar />
    </div>
    <div class="w-1/5 min-w-[24rem] max-w-[32rem] flex-shrink-0 print:hidden" />
  {:else}
    <label
      class="btn no-animation bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleSidebar = !toggleSidebar
        }}
      />
      <span class="swap-off icon-[fluent--navigation-20-regular] fill-current w-5 h-5"></span>
      <span class="swap-on icon-[fluent--dismiss-20-regular] fill-current w-5 h-5"></span>
    </label>
  {/if}
  {#if error - 200 < 100}
    <slot />
  {:else}
    <Error status={error} />
  {/if}
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10 print:hidden"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <Sidebar />
    </div>
  {/if}
</div>
