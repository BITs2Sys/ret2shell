<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { page } from '$app/stores'
  import type { Wiki } from '$lib/models/wiki'
  import { platform } from '$lib/stores/platform'
  import { getWiki } from '$lib/api/wiki'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { getUserInfo } from '$lib/api/user'
  import type { User } from '$lib/models/user'
  import Error from '$lib/blocks/Error.svelte'
  import { onDestroy } from 'svelte'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'

  let loading = true
  let error = 200
  let wiki: Wiki = {
    id: 0,
    title: $i18n.t('wiki.fetchingContent'),
    content: '',
    parent: 0,
    published_at: 0,
    updated_at: 0,
    author_id: 0,
  }

  function scrollToTop() {
    let pageTop = document.getElementById('page-top')
    pageTop?.scrollIntoView({ behavior: 'smooth' })
  }

  let user: User | undefined

  const unsubscribe = page.subscribe((page) => {
    if (!page.params['wiki']) return
    const arr = page.params['wiki'].split('/')

    const id = parseInt(arr[arr.length - 1]) || -1
    if (id === wiki.id) return
    loading = true
    if (id > 0) {
      getWiki(id)
        .then((data) => {
          wiki = data
          getUserInfo(data.author_id)
            .then((data) => {
              user = data
            })
            .catch(() => {
              user = {
                id: -1,
                name: $i18n.t('user.deletedAuthor'),
                email: '',
                intro: '',
                cover_path: null,
                institute_info: null,
                institute_id: null,
                permissions: [],
                hidden: false,
                banned: true,
              }
            })
          error = 200
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('wiki.fetchContentFailed')}: ${(err as AxiosError).response?.data}`)
          error = (err as AxiosError).response?.status || 500
        })
        .finally(() => {
          loading = false
        })
    } else {
      loading = false
    }
  })

  onDestroy(unsubscribe)
</script>

<svelte:head>
  <title>{wiki.title} - {$platform.name}</title>
</svelte:head>

<div class="flex-1 flex flex-col p-4 lg:p-6 items-center">
  {#if loading}
    <div class="h-16 flex flex-row justify-center items-center space-x-2">
      <span class="loading loading-spinner loading-sm" />
      <span class="text-base">{$i18n.t('wiki.fetchingContent')}</span>
    </div>
  {:else if error - 200 >= 100}
    <Error status={error} />
  {:else}
    <h1 class="text-3xl font-bold h-16 mt-12 flex justify-center items-center">
      {wiki.title}
    </h1>
    <div class="flex flex-row space-x-4 flex-wrap">
      <p>
        <span class="text-base opacity-80">{$i18n.t('wiki.author')}</span>
        :
        <span class="text-base opacity-80">{user?.name || $i18n.t('wiki.unknownAuthor')}</span>
      </p>
      <p>
        <span class="text-base opacity-80">{$i18n.t('wiki.publishedAt')}</span>
        :
        <span class="text-base opacity-80">{new Date(wiki.published_at * 1000).toLocaleString()}</span>
      </p>
      {#if wiki.published_at !== wiki.updated_at}
        <p>
          <span class="text-base opacity-80">{$i18n.t('wiki.updatedAt')}</span>
          :
          <span class="text-base opacity-80">{new Date(wiki.updated_at * 1000).toLocaleString()}</span>
        </p>
      {/if}
    </div>
    <RxArticle content={wiki.content} headingAnchors={true} class="p-6 pt-12" />
    <RxButton square size="lg" class="fixed bottom-28 right-6 print:hidden" on:click={scrollToTop}>
      <span class="icon-[fluent--chevron-up-20-regular] w-5 h-5"></span>
    </RxButton>
    <div class="h-32" />
  {/if}
</div>
