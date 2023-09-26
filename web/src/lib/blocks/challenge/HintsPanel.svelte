<script lang="ts">
  import { i18n } from '$lib/i18n'
  import type { Hint } from '$lib/models/hint'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'

  let clazz = ''
  export { clazz as class }
  $: classes = `w-full flex-1 relative overflow-hidden ${clazz}`
  export let hints: Hint[] = []
</script>

<div class={classes}>
  <div class="absolute w-full h-full">
    <OverlayScrollbarsComponent
      options={{
        scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
      }}
      class="w-full h-full relative print:hidden"
      defer
    >
      {#if hints.length === 0}
        <p class="w-full min-h-full flex-1 flex flex-row justify-center items-center font-bold opacity-60">
          {$i18n.t('challenges.noHint')}
        </p>
      {/if}
      {#each hints as item}
        <div
          class="flex flex-row border-b border-b-base-content/5 h-16 items-center px-4 space-x-2 hover:bg-base-content/5"
        >
          <span class="icon-[fluent--info-20-regular] w-6 h-6 text-info" />
          <span>{item.content}</span>
        </div>
      {/each}
    </OverlayScrollbarsComponent>
  </div>
</div>
