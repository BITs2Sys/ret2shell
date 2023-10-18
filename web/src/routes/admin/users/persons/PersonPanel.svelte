<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import type { User } from '$lib/models/user'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { createEventDispatcher } from 'svelte'
  import Info from './Info.svelte'
  import Statistics from './Statistics.svelte'
  import { i18n } from '$lib/i18n'
  import { blur } from 'svelte/transition'

  export let user: User | null
  export let loading = false

  let clazz = ''
  let activeTab = 'info'
  export { clazz as class }

  $: classes = `w-full flex flex-col overflow-hidden bg-neutral/20 backdrop-blur ${clazz}`
  const dispatch = createEventDispatcher()
</script>

<div class={classes}>
  <OverlayScrollbarsComponent
    options={{
      scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
    }}
    class="w-full h-full relative print:hidden"
    defer
  >
    {#if loading}
      <div
        class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
        transition:blur={{ amount: 20, duration: 300 }}
      >
        <span class="loading loading-spinner loading-sm" />
      </div>
    {/if}
    <div class="sticky top-0 h-16 min-h-16 border-b border-b-base-content/5 flex flex-row px-2 items-center space-x-2">
      <div class="flex-1 flex flex-row items-center space-x-2 px-4">
        <h1 class="text-base font-bold">{user?.name}</h1>
        <RxButton class="!ml-12" ghost on:click={() => (activeTab = 'info')} active={activeTab === 'info'}>
          {$i18n.t('admin.userInfo')}
        </RxButton>
        <RxButton ghost on:click={() => (activeTab = 'statistics')} active={activeTab === 'statistics'}>
          {$i18n.t('admin.userStatistics')}
        </RxButton>
      </div>
      <RxButton
        ghost
        level="error"
        class="join-item ml-0"
        on:click={() => {
          dispatch('close')
        }}
      >
        <span class="icon-[fluent--dismiss-20-regular] w-5 h-5"></span>
      </RxButton>
    </div>
    {#if activeTab === 'info'}
      <Info {user} />
    {:else if activeTab === 'statistics'}
      <Statistics {user} />
    {/if}
  </OverlayScrollbarsComponent>
</div>
