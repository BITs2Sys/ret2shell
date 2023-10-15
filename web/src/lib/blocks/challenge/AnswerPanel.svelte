<script lang="ts">
  import RxArticle from '$lib/components/RxArticle.svelte'
  import { i18n } from '$lib/i18n'
  import type { Answer } from '$lib/models/answer'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'

  let clazz = ''
  export { clazz as class }
  $: classes = `w-full flex-1 relative overflow-hidden ${clazz}`

  export let answer: Answer | null = null
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
      {#if !answer || answer.content.length === 0}
        <p class="w-full min-h-full flex-1 flex flex-row justify-center items-center font-bold opacity-60">
          {$i18n.t('playground.emptyContent')}
        </p>
      {:else}
        <div class="w-full flex flex-col items-center">
          <div class="flex flex-col w-full max-w-5xl px-6">
            <RxArticle class="mt-12" content={answer?.content || $i18n.t('playground.emptyContent')} />
            <div class="h-12" />
          </div>
        </div>
      {/if}
    </OverlayScrollbarsComponent>
  </div>
</div>
