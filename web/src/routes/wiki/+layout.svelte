<script lang="ts">
  import { fly } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'
  import Sidebar from './Sidebar.svelte'
  import { transformToWikiEntry, type WikiEntry } from '$lib/models/wiki'
  import { onDestroy, onMount } from 'svelte'
  import { getWikiList } from '$lib/api/wiki'
  import type { AxiosError } from 'axios'
  import { showMessage } from '$lib/stores/toast'
  import { i18n } from '$lib/i18n'
  import { page } from '$app/stores'
  import Error from '$lib/blocks/Error.svelte'

  let toggleSidebar = false
  let screenWidth: number
  let activeChains: number[] = []
  let wikiEntries: WikiEntry[] = []
  let loading = false
  let error = 200

  // give path:  /wiki/1/2/3/4 ...
  // we should loop through the path and request-and-insert the path children into the tree
  // if the path children returns not found or activeId do not exist in the path children,
  // we should show error page.
  async function recursiveInsertPath(path: number[]) {
    let currentId = 0
    let currentChildren: WikiEntry[] = wikiEntries
    for (let i = 0; i < path.length; i++) {
      currentId = path[i]
      // console.log(currentId)
      // console.log(currentChildren)
      let currentChild = currentChildren.find((item) => item.id === currentId)
      if (!currentChild) {
        loading = false
        return
      }
      const newChildren = await getWikiList(currentId)
      currentChild.children = transformToWikiEntry(newChildren)
      currentChildren = currentChild.children
    }
    wikiEntries = wikiEntries

    loading = false
  }

  onMount(() => {
    loading = true
    getWikiList()
      .then((data) => {
        wikiEntries = transformToWikiEntry(data)
        error = 200
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('wiki.fetchFailed')}: ${err.response?.data}`)
        error = (err as AxiosError).response?.status || 500
      })
      .finally(() => {
        if (!$page.params['wiki']) {
          loading = false
          return
        }
        const path = $page.params['wiki'].split('/').map((item) => {
          try {
            return parseInt(item)
          } catch (err) {
            return -1
          }
        })
        activeChains = path
        recursiveInsertPath(path)
      })
  })

  $: showSidebar = screenWidth > 1024 // lg

  const unsubscribe = page.subscribe((value) => {
    if (value.params['wiki']) {
      activeChains = value.params['wiki'].split('/').map((item) => {
        try {
          return parseInt(item)
        } catch (err) {
          return -1
        }
      })
    } else {
      activeChains = [-1]
    }
  })
  onDestroy(unsubscribe)
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row">
  {#if showSidebar}
    <div
      class="fixed w-1/5 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-neutral/20 backdrop-blur border-r border-r-base-content/10 flex flex-col print:hidden"
    >
      <Sidebar bind:wiki={wikiEntries} {activeChains} {loading} />
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
      class="fixed w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10 flex flex-col print:hidden"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <Sidebar bind:wiki={wikiEntries} {activeChains} {loading} />
    </div>
  {/if}
</div>
