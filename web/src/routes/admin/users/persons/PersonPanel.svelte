<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import type { User } from '$lib/models/user'
  import { createEventDispatcher } from 'svelte'
  import Info from './Info.svelte'
  import Statistics from './Statistics.svelte'
  import { i18n } from '$lib/i18n'
  import { blur } from 'svelte/transition'
  import type { Institute } from '$lib/models/institute'
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'

  export let user: User | null
  export let loading = false
  export let institutes: Institute[] = []

  let clazz = ''
  let activeTab = 'info'
  export { clazz as class }
  const dispatch = createEventDispatcher()
</script>

<ExtraPanel class={clazz} on:close={() => dispatch('close')}>
  <div class="flex-1 flex flex-row items-center space-x-2 px-4" slot="header">
    <h1 class="text-base font-bold">{user?.name}</h1>
    <RxButton class="!ml-12" ghost on:click={() => (activeTab = 'info')} active={activeTab === 'info'}>
      {$i18n.t('admin.userInfo')}
    </RxButton>
    <RxButton ghost on:click={() => (activeTab = 'statistics')} active={activeTab === 'statistics'}>
      {$i18n.t('admin.userStatistics')}
    </RxButton>
  </div>

  {#if activeTab === 'info'}
    <Info {user} {institutes} />
  {:else if activeTab === 'statistics'}
    <Statistics {user} />
  {/if}
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</ExtraPanel>
