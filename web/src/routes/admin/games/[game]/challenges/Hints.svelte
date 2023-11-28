<script lang="ts">
  import { createHint, deleteHint, getChallengeHints } from '$lib/api/challenge'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import type { Hint } from '$lib/models/hint'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onMount } from 'svelte'

  export let challenge: Challenge
  let loading = false
  let hints: Hint[] = []
  let newHint: Hint = {
    id: 0,
    created_at: 0,
    challenge_id: 0,
    content: '',
  }

  function fetchHints() {
    loading = true
    getChallengeHints(challenge.id)
      .then((res) => {
        hints = res
        // console.log(hints)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('submissions.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  function handleDeleteHint(item: Hint) {
    deleteHint(challenge.id, item.id)
      .then(() => {
        showMessage('success', $i18n.t('hint.deleteSuccess'), 5000)
        fetchHints()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('hint.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  function handleCreateHint() {
    createHint(challenge.id, newHint)
      .then(() => {
        showMessage('success', $i18n.t('hint.createSuccess'), 5000)
        newHint = {
          id: 0,
          created_at: 0,
          challenge_id: 0,
          content: '',
        }
        fetchHints()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('hint.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  onMount(() => {
    fetchHints()
  })
</script>

<div class="w-full flex flex-1 flex-col">
  <div class="w-full h-full">
    <OverlayScrollbarsComponent
      options={{
        scrollbars: {
          theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light',
          autoHide: 'scroll',
        },
      }}
      class="w-full h-full relative p-6 print:hidden"
      defer
    >
      <div class="flex flex-col space-y-4">
        <div class="flex flex-row space-x-2">
          <RxInput bind:value={newHint.content} class="w-full" placeholder={$i18n.t('hint.new')} />
          <RxButton ghost on:click={handleCreateHint}>
            <span class="icon-[fluent--add-20-regular] w-5 h-5 text-info"></span>
          </RxButton>
        </div>
        {#if hints.length === 0}
          <p class="w-full flex-1 flex flex-row justify-center items-center font-bold opacity-60">
            {$i18n.t('challenges.noHint')}
          </p>
        {/if}
        {#each hints as item}
          <div class="flex flex-row border-b border-b-base-content/5 h-16 items-center pl-4 space-x-2 w-full">
            <span class="icon-[fluent--info-20-regular] w-5 h-5 text-info flex-shrink-0" />
            <div class="text-base overflow-hidden w-full">
              <span>{item.content}</span>
            </div>
            <RxButton
              ghost
              on:click={() => {
                handleDeleteHint(item)
              }}
            >
              <span class="icon-[fluent--delete-20-regular] w-5 h-5 text-error"></span>
            </RxButton>
          </div>
        {/each}
      </div>
    </OverlayScrollbarsComponent>
  </div>
</div>
