<script lang="ts">
  import RxButton from './RxButton.svelte'

  export let total = 0
  export let page = 1

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
  <div class="flex justify-center mt-4 mb-12">
    <div class="join">
      {#each paginationKeys as key}
        <RxButton
          class="join-item ml-0"
          active={key === page}
          on:click={() => {
            page = key
          }}
          disabled={key === -1}
        >
          {key === -1 ? '..' : key}
        </RxButton>
      {/each}
    </div>
  </div>
{/if}
