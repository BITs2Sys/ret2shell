<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { blur } from 'svelte/transition'

  export let loading = true
  export let src: string
  // object-contain object-cover object-fill
  export let fit: 'contain' | 'cover' | 'fill' = 'cover'

  let clazz = ''
  export { clazz as class }

  $: classes = ['overflow-hidden relative', clazz].filter(Boolean).join(' ')

  let loadingCover = true
  let hasError = false

  function stateWatcher(node: HTMLImageElement) {
    const onload = () => {
      // console.log('src loaded:', node.complete)
      node.complete ? (loadingCover = false) : (loadingCover = true)
      hasError = false
    }
    const onerror = () => {
      // console.log('src load failed:', node.complete)
      loadingCover = false
      hasError = true
    }
    node.addEventListener('load', onload)
    node.addEventListener('error', onerror)
  }

  $: {
    if (src) {
      // console.log('src changed:', src)
      loadingCover = true
      hasError = false
    }
  }
</script>

<div class={classes} {...$$restProps}>
  <img class={`object-${fit} w-full h-full`} alt={$i18n.t('form.imageBroken')} {src} use:stateWatcher />
  {#if loading || loadingCover}
    <div
      class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center bg-neutral"
      transition:blur={{ amount: 20 }}
    >
      <span class="loading" />
    </div>
  {/if}
  <slot />
</div>
