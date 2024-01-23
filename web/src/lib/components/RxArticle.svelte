<script lang="ts">
  import { i18n } from '$lib/i18n'
  import '$lib/styles/article.scss'

  export let content: string
  export let showRenderTips = true
  export let headingAnchors: boolean = false
  $: contentRendered = render(content)
  let ready = false
  let contentHtml = ''
  let clazz = ''
  export { clazz as class }
  $: classes = `prose max-w-5xl w-full ${clazz}`

  function scrollToView() {
    setTimeout(() => {
      if (location.hash.replace('#', '').length > 0)
        document.getElementById(decodeURI(location.hash.replace('#', '')))?.scrollIntoView({ behavior: 'smooth' })
    }, 100)
  }

  $: {
    contentRendered.then((val) => {
      ready = true
      contentHtml = val
      scrollToView()
    })
  }

  const render = async (content: string) => {
    ready = false
    let { MarkTo } = await import('$lib/markto')
    let dompurify = await import('isomorphic-dompurify')
    const markTo = new MarkTo()
    await markTo.init({ type: 'html', options: { prism: true, katex: true, headingAnchors } })
    return dompurify.sanitize((await markTo.render(content)) as string)
  }
</script>

<article class={classes}>
  {#if showRenderTips && !ready}
    <span class="loading loading-spinner loading-sm" />
    <span>{$i18n.t('wiki.rendering')}</span>
  {:else}
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html contentHtml}
  {/if}
</article>
