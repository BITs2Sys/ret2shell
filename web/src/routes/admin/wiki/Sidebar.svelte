<script lang="ts">
  import { page } from '$app/stores'
  import { getWikiList } from '$lib/api/wiki'
  import WikiTree from '$lib/blocks/WikiTree.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { transformToWikiEntry, type WikiEntry } from '$lib/models/wiki'
  import { showMessage } from '$lib/stores/toast'
  import { onDestroy, onMount } from 'svelte'

  let wiki: WikiEntry[] = []
  let activeChains: number[] = []
  let loading: boolean

  async function fetchChildren(id: number) {
    return transformToWikiEntry(await getWikiList(id))
  }

  // give path:  /wiki/1/2/3/4 ...
  // we should loop through the path and request-and-insert the path children into the tree
  // if the path children returns not found or activeId do not exist in the path children,
  // we should show error page.
  async function recursiveInsertPath(path: number[]) {
    let currentId = 0
    const root = transformToWikiEntry(await getWikiList())
    const replace = (
      a: Array<WikiEntry>,
      b: Array<WikiEntry>,
      predicate = (a: WikiEntry, b: WikiEntry) => a.id === b.id
    ) => {
      return b.map((item) => {
        const index = a.findIndex((i) => predicate(i, item))
        if (index === -1) {
          return item
        }
        return a[index]
      })
    }
    // merge root into wiki
    wiki = replace(wiki, root)
    let currentChildren: WikiEntry[] = wiki

    for (let i = 0; i < path.length; i++) {
      currentId = path[i]
      let currentChild = currentChildren.find((item) => item.id === currentId)
      if (!currentChild) {
        loading = false
        return
      }
      const newChildren = await getWikiList(currentId)
      currentChild.children = replace(currentChild.children, transformToWikiEntry(newChildren))
      currentChildren = currentChild.children
    }
    wiki = wiki

    loading = false
  }

  onMount(() => {
    loading = true
    getWikiList()
      .then((data) => {
        wiki = transformToWikiEntry(data)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('wiki.fetchFailed')}: ${err.response?.data}`)
        // error = (err as AxiosError).response?.status || 500
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
      recursiveInsertPath(activeChains)
    } else {
      activeChains = []
    }
  })
  onDestroy(unsubscribe)
</script>

<RxLink href="/admin/wiki/0#create" class="sticky top-4 z-10 bg-neutral">
  <span class="icon-[fluent--add-20-regular] w-5 h-5"></span>
  <span>{$i18n.t('admin.create')}</span>
</RxLink>
{#if loading}
  <div class="flex flex-row justify-center items-center h-16 space-x-2">
    <span class="loading loading-spinner loading-sm" />
    <span class="text-base">{$i18n.t('wiki.fetchingList')}</span>
  </div>
{:else if wiki.length === 0}
  <p class="text-base font-semibold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
{:else}
  <WikiTree bind:tree={wiki} {activeChains} addrPrefix="/admin/wiki" {fetchChildren} manageBtn={true} />
{/if}
