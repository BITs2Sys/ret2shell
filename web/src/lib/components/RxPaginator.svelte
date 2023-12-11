<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import RxButton from './RxButton.svelte'

  export let total = 0
  export let page = 1

  const dispatch = createEventDispatcher()

  // p: page, t: total
  function calculatePagination(p: number, t: number) {
    const buttons = [] as number[]
    buttons.push(1)
    if (p > 3) buttons.push(-1)
    for (let i = 2; i <= t - 1; i++) {
      if (i > p - 2 && i < p + 2) buttons.push(i)
    }
    if (p < t - 2) buttons.push(-1)
    if (t > 1) buttons.push(t)
    return buttons
  }

  $: paginationKeys = calculatePagination(page, total)
</script>

{#if total >= 1}
  <div class="flex justify-center items-center space-x-2 mt-4 mb-12">
    <RxButton
      on:click={() => {
        if (page - 1 <= 0) return
        page = page - 1
        dispatch('select-page', page)
      }}
      square
    >
      <span class="icon-[fluent--chevron-left-20-regular]"></span>
    </RxButton>
    {#each paginationKeys as key}
      {#if key !== -1}
        <RxButton
          active={key === page}
          on:click={() => {
            page = key
            dispatch('select-page', key)
          }}
          square
        >
          {key}
        </RxButton>
      {:else}
        <span class="opacity-60">...</span>
      {/if}
    {/each}
    <RxButton
      on:click={() => {
        if (page + 1 > total) return
        page = page + 1
        dispatch('select-page', page)
      }}
      square
    >
      <span class="icon-[fluent--chevron-right-20-regular]"></span>
    </RxButton>
  </div>
{/if}
