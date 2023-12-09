<script lang="ts">
  import { createTag, deleteTag, getTagList } from '$lib/api/v1/challenge'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Tag } from '$lib/models/challenge'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import { blur } from 'svelte/transition'

  let tags: Tag[] = []
  let content = ''
  let loading = false

  function refreshTags() {
    loading = true
    getTagList()
      .then((res) => {
        tags = res
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('tag.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  function handleDeleteTag(id: number) {
    deleteTag(id)
      .then(() => {
        refreshTags()
        showMessage('success', $i18n.t('tag.deleteSuccess'), 5000)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('tag.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  function handleCreateTag() {
    createTag({
      id: -1,
      name: content,
    })
      .then(() => {
        refreshTags()
        content = ''
        showMessage('success', $i18n.t('tag.createSuccess'), 5000)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('tag.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  onMount(() => {
    refreshTags()
  })
</script>

<svelte:head><title>{$i18n.t('admin.routes.tags')} - {$platform.name}</title></svelte:head>
<div class="flex-1 relative flex justify-center">
  <div class="flex-1 flex flex-col max-w-5xl p-4 lg:p-6">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('admin.routes.tags')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <div class="flex flex-row my-4">
      <RxInput class="flex-1" icon="icon-[fluent--add-20-regular]" bind:value={content}>
        <RxButton class="join-item ml-0" on:click={handleCreateTag}>
          <span class="icon-[fluent--add-20-regular] w-5 h-5"></span>
          <span>{$i18n.t('admin.create')}</span>
        </RxButton>
      </RxInput>
    </div>
    <div class="flex flex-row flex-wrap">
      {#each tags as tag}
        <div class="p-2 pl-4 m-2 flex flex-row items-center space-x-2 bg-base-content/5 backdrop-blur rounded-box">
          <span class="icon-[fluent--tag-20-regular]"></span>
          <span>{tag.name}</span>
          <RxButton
            size="sm"
            square
            ghost
            on:click={() => {
              handleDeleteTag(tag.id)
            }}
          >
            <span class="icon-[fluent--dismiss-20-regular]"></span>
          </RxButton>
        </div>
      {/each}
    </div>
  </div>
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</div>
