<script lang="ts">
  import { getWikiList } from '$lib/api/wiki'
  import RxTree from '$lib/components/RxTree.svelte'
  import { i18n } from '$lib/i18n'
  import { transformToWikiEntry, type WikiEntry } from '$lib/models/wiki'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'

  export let wiki: WikiEntry[]
  export let activeChains: number[] = []
  export let loading: boolean

  async function fetchChildren(id: number) {
    return transformToWikiEntry(await getWikiList(id))
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
    class="h-16 bg-base-100 border-b border-b-base-content/5 flex flex-row items-center justify-center space-x-2 p-4 font-bold sticky top-0 z-10"
  >
    <span class="w-6 h-6 icon-[fluent--hat-graduation-16-regular]" />
    <span>
      {$i18n.t('wiki.sideToc')}
    </span>
  </h1>
  {#if loading}
    <div class="flex flex-row justify-center items-center h-16 space-x-2">
      <span class="loading loading-spinner" />
      <span class="text-base">{$i18n.t('wiki.fetchingList')}</span>
    </div>
  {:else}
    {#if wiki.length === 0}
      <p class="text-base font-semibold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
    {/if}
    <div class="p-4">
      <RxTree bind:tree={wiki} {activeChains} addrPrefix="/wiki" {fetchChildren} />
    </div>
  {/if}
</OverlayScrollbarsComponent>
