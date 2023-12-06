<script lang="ts">
  import { page } from '$app/stores'
  import { getChallenge } from '$lib/api/v1/challenge'
  import Error from '$lib/blocks/Error.svelte'
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { createEventDispatcher, onDestroy } from 'svelte'
  import Statistics from './Statistics.svelte'
  import Info from './Info.svelte'
  import Repo from './Repo.svelte'
  import Workflows from './Workflows.svelte'
  import Submissions from './Submissions.svelte'
  import Instances from './Instances.svelte'
  import { blur } from 'svelte/transition'
  import Hints from './Hints.svelte'
  import Answer from './Answer.svelte'
  const dispatch = createEventDispatcher()

  let clazz = ''
  export { clazz as class }

  let currentChallenge: Challenge | null = null
  let error = 200
  let loading = false
  let activeTab = 'statistics'
  export let tags: Tag[]

  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      const challengeId = parseInt(val.url.hash.replace('#', ''))
      if (isNaN(challengeId)) {
        error = 404
        showMessage('error', $i18n.t('challenge.notFound'), 5000)
      }
      loading = true
      activeTab = 'statistics'
      getChallenge(challengeId)
        .then((res) => {
          currentChallenge = res
          loading = false
        })
        .catch((err) => {
          error = (err as AxiosError).response?.status || 500
          showMessage('error', `${$i18n.t('challenge.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          loading = false
        })
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>

{#if error < 300}
  <ExtraPanel on:close={() => dispatch('close')} class={clazz}>
    <div class="flex-1 flex flex-row items-center space-x-2 px-4" slot="header">
      {#if loading}
        <h1 class="text-base font-bold">
          <span class="loading loading-spinner loading-sm"></span>
          <span>{$i18n.t('admin.loadingChallenge')}</span>
        </h1>
      {:else}
        <h1 class="text-base font-bold max-w-[16rem] overflow-hidden truncate">
          {currentChallenge?.name}
        </h1>
      {/if}
      <RxButton
        class="!ml-12"
        ghost
        on:click={() => (activeTab = 'statistics')}
        active={activeTab === 'statistics'}
        disabled={loading}
      >
        {$i18n.t('admin.challengeStatistics')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'info')} active={activeTab === 'info'} disabled={loading}>
        {$i18n.t('admin.challengeInfo')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'repo')} active={activeTab === 'repo'} disabled={loading}>
        {$i18n.t('admin.challengeRepo')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'workflows')} active={activeTab === 'workflows'} disabled={loading}>
        {$i18n.t('admin.challengeWorkflows')}
      </RxButton>
      <RxButton
        ghost
        on:click={() => (activeTab = 'submissions')}
        active={activeTab === 'submissions'}
        disabled={loading}
      >
        {$i18n.t('admin.challengeSubmissions')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'instances')} active={activeTab === 'instances'} disabled={loading}>
        {$i18n.t('admin.challengeInstances')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'hints')} active={activeTab === 'hints'} disabled={loading}>
        {$i18n.t('admin.challengeHints')}
      </RxButton>
      <RxButton ghost on:click={() => (activeTab = 'answer')} active={activeTab === 'answer'} disabled={loading}>
        {$i18n.t('admin.challengeAnswer')}
      </RxButton>
    </div>
    {#if currentChallenge}
      {#if activeTab === 'statistics'}
        <Statistics challenge={currentChallenge} />
      {:else if activeTab === 'info'}
        <Info {tags} challenge={currentChallenge} />
      {:else if activeTab === 'repo'}
        <Repo challenge={currentChallenge} />
      {:else if activeTab === 'workflows'}
        <Workflows challenge={currentChallenge} />
      {:else if activeTab === 'submissions'}
        <Submissions challenge={currentChallenge} />
      {:else if activeTab === 'instances'}
        <Instances challenge={currentChallenge} />
      {:else if activeTab === 'hints'}
        <Hints challenge={currentChallenge} />
      {:else if activeTab === 'answer'}
        <Answer challenge={currentChallenge} />
      {/if}
    {:else}
      <div class="w-full h-full flex flex-col justify-center items-center">
        <h2 class="font-bold text-base opacity-60">{$i18n.t('admin.noChallengeLoaded')}</h2>
      </div>
    {/if}
    {#if loading}
      <div
        class="w-full h-full absolute top-0 left-0 flex flex-col items-center justify-center backdrop-blur"
        transition:blur={{ amount: 20, duration: 300 }}
      >
        <span class="loading loading-spinner loading-sm"></span>
      </div>
    {/if}
  </ExtraPanel>
{:else}
  <Error status={error}></Error>
{/if}
