<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import { transformToWikiEntry, type WikiEntry } from '$lib/models/wiki'
  import { onDestroy, onMount } from 'svelte'
  import { getWikiList } from '$lib/api/v1/wiki'
  import type { AxiosError } from 'axios'
  import { showMessage } from '$lib/stores/toast'
  import { i18n } from '$lib/i18n'
  import { page } from '$app/stores'
  import Error from '$lib/blocks/Error.svelte'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import { writable } from 'svelte/store'

  let activeChains: number[] = []
  let wikiEntries = writable<WikiEntry[]>([])
  let loading = false
  let error = 200

  // give path:  /wiki/1/2/3/4 ...
  // we should loop through the path and request-and-insert the path children into the tree
  // if the path children returns not found or activeId do not exist in the path children,
  // we should show error page.
  async function recursiveInsertPath(path: number[]) {
    let currentId = 0
    let clonedEntries = $wikiEntries
    let currentChildren = clonedEntries
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
    wikiEntries.update((val) => {
      val = clonedEntries
      return val
    })

    loading = false
  }

  onMount(() => {
    loading = true
    getWikiList()
      .then((data) => {
        $wikiEntries = transformToWikiEntry(data)
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

<SidebarLayout
  leftSidebar={Sidebar}
  leftProps={{
    loading,
    wiki: wikiEntries,
    activeChains,
  }}
>
  {#if error - 200 < 100}
    <slot />
  {:else}
    <Error status={error} />
  {/if}
</SidebarLayout>
