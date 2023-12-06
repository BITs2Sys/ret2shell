<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { getAnnouncementList } from '$lib/api/v1/announcement'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import RxPaginator from '$lib/components/RxPaginator.svelte'
  import RxLink from '$lib/components/RxLink.svelte'

  let announcements: Announcement[] = []
  let loading = false
  let page = 1
  let perPage = 12
  let total = 0

  function refresh(page: number, perPage: number) {
    loading = true
    getAnnouncementList(page, perPage)
      .then((data) => {
        announcements = data.announcements
        total = data.total
        loading = false
      })
      .catch((error) => {
        showMessage('error', `${$i18n.t('announcements.fetchFailed')}: ${(error as AxiosError).response?.data}`, 5000)
      })
  }

  $: {
    refresh(page, perPage)
  }
</script>

<svelte:head><title>{$i18n.t('announcements.title')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('announcements.title')}</h1>
      <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
    </div>

    {#if loading}
      <div class="flex flex-row justify-center items-center h-16 space-x-2">
        <span class="loading loading-spinner loading-sm" />
        <span class="text-base">{$i18n.t('announcements.fetchingList')}</span>
      </div>
    {:else}
      {#if announcements.length === 0}
        <p class="text-base font-semibold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
      {/if}
      <div class="flex-1 flex flex-col space-y-2 overflow-scroll mt-4">
        {#each announcements as item}
          <div class="border-b border-b-base-content/5 w-full">
            <RxLink class="w-full" justify="start" ghost href={`/announcements/${item.id}`}>
              <span class={`icon-[fluent--megaphone-20-regular] w-5 h-5 ${item.pinned ? 'text-error' : ''}`} />
              <span class="text-base">{item.title}</span>
              <div class="flex-1" />
              <span class="text-base opacity-60">
                {new Date(item.published_at * 1000).toLocaleDateString('default', {
                  year: 'numeric',
                  day: '2-digit',
                  month: '2-digit',
                })}
              </span>
            </RxLink>
          </div>
        {/each}
      </div>
      <RxPaginator bind:page {total}></RxPaginator>
    {/if}
  </div>
</div>
