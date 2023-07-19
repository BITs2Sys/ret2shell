<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { createEventDispatcher } from 'svelte'

  export let loading = true
  export let src: string

  let clazz = ''
  export { clazz as class }

  $: classes = ['overflow-hidden relative', clazz].filter(Boolean).join(' ')

  let loadingCover = true

  const dispatcher = createEventDispatcher()

  const handleLoad = () => {
    loadingCover = false
    dispatcher('loaded')
  }
</script>

<div class={classes} {...$$restProps}>
  {#if !loading}
    <img class="object-cover w-full h-full" alt={$i18n.t('global.imageBroken')} {src} on:load={handleLoad} />
  {/if}
  {#if loading || loadingCover}
    <div class="w-full h-full flex flex-col justify-center items-center">
      <span class="loading" />
    </div>
  {/if}
  <slot />
</div>
