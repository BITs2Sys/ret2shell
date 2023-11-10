<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { createEventDispatcher } from 'svelte'
  import type { WriteUpWithInfo } from '$lib/models/write_up'

  export let writeup: WriteUpWithInfo
  export let loading = false
  export let submitting = false

  let clazz = ''
  export { clazz as class }

  const dispatch = createEventDispatcher()

  function scrollToTop() {
    let pageTop = document.getElementById('page-top')
    pageTop?.scrollIntoView({ behavior: 'smooth' })
  }
</script>

<svelte:head><title>{writeup.title} - {$platform.name}</title></svelte:head>

<div class={clazz}>
  <div class="flex-1 flex flex-col self-center w-full p-4 lg:p-6 items-center">
    {#if loading}
      <div class="h-16 flex flex-row justify-center items-center space-x-2">
        <span class="loading loading-spinner loading-sm" />
        <span class="text-base">{$i18n.t('writeup.fetchingContent')}</span>
      </div>
    {:else}
      <h1 class="text-3xl font-bold h-16 mt-12 flex justify-center items-center">
        {writeup.title}
      </h1>
      <div class="flex flex-row space-x-4 flex-wrap justify-center">
        <p>
          <span class="text-base opacity-80">{$i18n.t('writeup.author_name')}</span>
          :
          <span class="text-base opacity-80">{writeup?.author_name || $i18n.t('writeup.unknownAuthor')}</span>
        </p>
        <p>
          <span class="text-base opacity-80">{$i18n.t('writeup.team_name')}</span>
          :
          <span class="text-base opacity-80">{writeup?.team_name || $i18n.t('writeup.unknownAuthor')}</span>
        </p>
        {#if writeup.published_at === writeup.updated_at}
          <p>
            <span class="text-base opacity-80">{$i18n.t('writeup.publishedAt')}</span>
            :
            <span class="text-base opacity-80">{new Date(writeup.published_at * 1000).toLocaleString()}</span>
          </p>
        {:else}
          <p>
            <span class="text-base opacity-80">{$i18n.t('writeup.updatedAt')}</span>
            :
            <span class="text-base opacity-80">{new Date(writeup.updated_at * 1000).toLocaleString()}</span>
          </p>
        {/if}
      </div>
      <RxArticle content={writeup.content} headingAnchors={true} class="p-6 pt-12" />
      <RxButton
        square
        size="lg"
        class="fixed bottom-56 right-6 print:hidden"
        disabled={loading || submitting}
        on:click={() => dispatch('audit')}
      >
        {#if writeup.hidden}
          <span class="icon-[fluent--checkmark-20-regular] w-5 h-5"></span>
        {:else}
          <span class="icon-[fluent--dismiss-20-regular] w-5 h-5"></span>
        {/if}
      </RxButton>
      <RxButton
        square
        size="lg"
        class="fixed bottom-28 right-6 print:hidden"
        disabled={loading || submitting}
        on:click={scrollToTop}
      >
        <span class="icon-[fluent--chevron-up-20-regular] w-5 h-5"></span>
      </RxButton>
      <RxButton
        square
        size="lg"
        class="fixed bottom-6 right-6 print:hidden"
        disabled={loading || submitting}
        on:click={() => history.back()}
      >
        <span class="icon-[fluent--arrow-hook-down-left-20-regular] w-5 h-5"></span>
      </RxButton>
      <div class="h-32" />
    {/if}
  </div>
</div>
