<script lang="ts">
  import { i18n } from '$lib/i18n'
  import type { WikiEntry } from '$lib/models/wiki'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import RxButton from './RxButton.svelte'
  import RxLink from './RxLink.svelte'

  // pl-0 pl-4 pl-8 pl-12 pl-16
  export let depth = 0

  export let tree: WikiEntry[] = []
  export let treeExpandedRecord: Record<number, boolean> = {}
  export let treeLoadingRecord: Record<number, boolean> = {}
  export let activeId: number
  export let activeParentId: number | null = null
  export let childReportedParentId: number | null = null
  export let treeNoChildrenRecord: Record<number, boolean> = {}
  export let fetchChildren: (id: number) => Promise<WikiEntry[]> = () => Promise.resolve([])

  function handleLoadingChildItems(id: number) {
    if (tree.find((item) => item.id === id)?.children?.length || 0 > 0) {
      treeExpandedRecord[id] = !!!treeExpandedRecord[id]
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
  }

  $: {
    if (activeId) {
      let item = tree.find((item) => item.id === activeId)
      if (item) {
        activeParentId = item.parent
      }
    }
  }

  $: {
    if (childReportedParentId) {
      treeExpandedRecord[childReportedParentId] = true
      activeParentId = tree[childReportedParentId].parent
    }
  }
</script>

<ul class={`pl-${depth * 4}`}>
  {#each tree as item}
    <li>
      <div class="join w-full">
        <RxLink bold={false} ghost class="flex-1 join-item" justify="start" href={item.addr}>
          <span class="icon-[fluent--folder-20-regular] w-6 h-6 flex-shrink-0" />
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
            <span
              class="icon-[fluent--chevron-down-16-regular] w-6 h-6 flex-shrink-0 transition-all {treeExpandedRecord[
                item.id
              ]
                ? ' rotate-0'
                : '-rotate-90'}"
            />
          </RxButton>
        {/if}
      </div>
      {#if treeExpandedRecord[item.id]}
        {#if item.children}
          <svelte:self
            bind:tree={item.children}
            {fetchChildren}
            bind:activeParentId={childReportedParentId}
            bind:activeId
            depth={depth + 1}
          />
        {/if}
      {/if}
    </li>
  {/each}
</ul>
