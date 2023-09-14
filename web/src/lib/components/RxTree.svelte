<script lang="ts">
  import { i18n } from '$lib/i18n'
  import type { WikiEntry } from '$lib/models/wiki'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import RxButton from './RxButton.svelte'
  import RxLink from './RxLink.svelte'
  import { onMount } from 'svelte'
  
  // pl-0 pl-4 pl-8 pl-12 pl-16
  export let depth = 0

  export let tree: WikiEntry[] = []
  export let activeChains: number[] = []
  export let addrPrefix = ''
  let itemActiveId = 0
  export let treeExpandedRecord: Record<number, boolean> = {}
  export let treeLoadingRecord: Record<number, boolean> = {}
  export let treeNoChildrenRecord: Record<number, boolean> = {}
  export let fetchChildren: (id: number) => Promise<WikiEntry[]> = () => Promise.resolve([])

  function handleLoadingChildItems(id: number) {
    if (tree.find((item) => item.id === id)?.children?.length || 0 > 0) {
      treeExpandedRecord[id] = !treeExpandedRecord[id]
      treeExpandedRecord = treeExpandedRecord
      return
    }
    treeLoadingRecord[id] = true
    fetchChildren(id)
      .then((children) => {
        tree.find((item) => item.id === id)!.children = children
        treeExpandedRecord[id] = true
        treeLoadingRecord[id] = false
        if (children.length === 0) {
          treeNoChildrenRecord[id] = true
        }
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('wiki.fetchFailed')}: ${(err as AxiosError).response?.data}`)
        treeLoadingRecord[id] = false
        treeNoChildrenRecord[id] = true
      })
      .finally(() => {
        treeLoadingRecord = treeLoadingRecord
        treeExpandedRecord = treeExpandedRecord
        treeNoChildrenRecord = treeNoChildrenRecord
      })
  }

  function reactiveActiveChains(chains: number[]) {
    if (chains && chains.length > 0) {
      // if some id in chains also in the tree, we should expand it
      let parentChains = chains.slice(0, chains.length - 1)
      parentChains.forEach((id) => {
        if (tree.find((item) => item.id === id)) {
          treeExpandedRecord[id] = true
        }
        treeExpandedRecord = treeExpandedRecord
      })
      // console.log(chains)
      if (chains.length > 0) itemActiveId = chains[chains.length - 1]
    }
  }

  onMount(() => {
    reactiveActiveChains(activeChains)
  })

  $: reactiveActiveChains(activeChains)
</script>

<ul
  class={`pl-${depth * 4} relative ${
    depth > 0 ? 'before:border-l-2 before:absolute before:h-full before:border-l-base-content/10' : ''
  }`}
>
  {#each tree as item}
    <li>
      <div class="join w-full overflow-hidden">
        <RxLink
          exactlyMatched
          bold={false}
          ghost
          class="flex-1 join-item overflow-hidden"
          justify="start"
          href={`${addrPrefix}/${item.id}`}
        >
          {#if depth === 0}
            <span class="icon-[fluent--notebook-20-regular] w-6 h-6 flex-shrink-0" />
          {:else}
            <span class="icon-[fluent--notepad-20-regular] w-6 h-6 flex-shrink-0" />
          {/if}
          <span class="text-ellipsis overflow-hidden whitespace-nowrap flex-1 text-left">{item.title}</span>
        </RxLink>
        {#if !treeNoChildrenRecord[item.id]}
          <RxButton
            class="join-item"
            ghost
            loading={treeLoadingRecord[item.id]}
            on:click={() => {
              handleLoadingChildItems(item.id)
            }}
          >
            {#if !treeLoadingRecord[item.id]}
              <span
                class="icon-[fluent--chevron-down-16-regular] w-5 h-5 flex-shrink-0 transition-all {treeExpandedRecord[
                  item.id
                ]
                  ? ' rotate-0'
                  : '-rotate-90'}"
              />
            {/if}
          </RxButton>
        {/if}
      </div>
      {#if treeExpandedRecord[item.id]}
        {#if item.children}
          <svelte:self
            bind:tree={item.children}
            {fetchChildren}
            {activeChains}
            depth={depth + 1}
            addrPrefix={`${addrPrefix}/${item.id}`}
          />
        {/if}
      {/if}
    </li>
  {/each}
</ul>
