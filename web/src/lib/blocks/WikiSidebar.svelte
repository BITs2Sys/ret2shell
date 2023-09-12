<script lang="ts">
  import { getWikiList } from '$lib/api/wiki'
  import RxTree from '$lib/components/RxTree.svelte'
  import { i18n } from '$lib/i18n'
  import { transformToWikiEntry, type WikiEntry } from '$lib/models/wiki'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onMount } from 'svelte'
  import { page } from '$app/stores'

  export let wiki: WikiEntry[]
  export let activeId: number
  export let loading: boolean

  onMount(() => {
    loading = true
    getWikiList()
      .then((data) => {
        wiki = transformToWikiEntry(data)
      })
      .catch((err) => {
        let error = err as AxiosError
        showMessage('error', `${$i18n.t('wiki.fetchFailed')}: ${error.response?.data}`)
      })
      .finally(() => {
        loading = false
      })

    tryExpandActive()
  })

  function tryExpandActive() {
    // get active id (number) from url /wiki/{wiki}
    // if active id is existed, expand the tree
    if ('wiki' in $page.params) {
      let activeId = parseInt($page.params.wiki)
      if (activeId) {
        // using `getWikiList(parent_id)` to get the parent of active id, and recursively get all the parent until
        // the parent is null
        // then, we should find the top parent item in the initial tree, then insert trees we have requested before
        // in a recursive way
        
      }
    }
  }
</script>

<OverlayScrollbarsComponent
  options={{
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
  }}
  class="w-full h-full flex flex-col relative print:hidden"
  defer
>
  <h1
    class="h-16 bg-base-100 border-b border-b-base-content/5 flex flex-row items-center justify-center space-x-2 p-4 font-bold sticky top-0"
  >
    <span class="w-6 h-6 icon-[fluent--hat-graduation-16-regular]" />
    <span>
      {$i18n.t('wiki.sideToc')}
    </span>
  </h1>
  {#if loading}
    <div class="flex flex-row justify-center items-center h-16">
      <span class="loading loading-spinner" />
      <span class="text-base">{$i18n.t('wiki.fetchingList')}</span>
    </div>
  {/if}
  <div class="p-4">
    <RxTree bind:tree={wiki} bind:activeId />
  </div>
</OverlayScrollbarsComponent>
