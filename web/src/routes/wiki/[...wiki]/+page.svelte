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
  import '$lib/styles/article.scss'

  let loading = true
  let wiki: Wiki = {
    id: 0,
    title: $i18n.t('wiki.fetchingContent'),
    content: '',
    parent: 0,
    published_at: 0,
    updated_at: 0,
    author_id: 0,
  }

  let user: User | undefined

  const render = async (content: string) => {
    let { MarkTo } = await import('$lib/markto')
    let dompurify = await import('isomorphic-dompurify')
    const markTo = new MarkTo()
    await markTo.init({ type: 'html', options: { prism: true, katex: true } })
    return dompurify.sanitize((await markTo.render(content)) as string)
  }

  let contentRendered: Promise<string> = render(wiki.content)

  function scrollToView() {
    setTimeout(() => {
      document.getElementById(decodeURI(location.hash.replace('#', '')))?.scrollIntoView({ behavior: 'smooth' })
    }, 100)
  }

  page.subscribe((page) => {
    if (!page.params['wiki']) return
    const arr = page.params['wiki'].split('/')

    const id = parseInt(arr[arr.length - 1]) || -1
    if (id === wiki.id) return
    loading = true
    if (id > 0) {
      getWiki(id)
        .then((data) => {
          wiki = data
          contentRendered = render(wiki.content)
          contentRendered.then(() => {
            scrollToView()
          })
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
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('wiki.fetchFailed')}: ${(err as AxiosError).response?.data}`)
        })
        .finally(() => {
          loading = false
        })
    } else {
      loading = false
    }
  })
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
  {:else}
      <h1 class="text-3xl font-bold h-16 mt-12 flex justify-center items-center">
        {wiki.title}
      </h1>
      <div class="flex flex-row space-x-4 flex-wrap">
        <p>
          <span class="text-base opacity-80">{$i18n.t('wiki.author')}</span>:
          <span class="text-base opacity-80">{user?.name || $i18n.t('wiki.unknownAuthor')}</span>
        </p>
        <p>
          <span class="text-base opacity-80">{$i18n.t('wiki.publishedAt')}</span>:
          <span class="text-base opacity-80">{new Date(wiki.published_at * 1000).toLocaleString()}</span>
        </p>
        {#if wiki.published_at !== wiki.updated_at}
          <p>
            <span class="text-base opacity-80">{$i18n.t('wiki.updatedAt')}</span>:
            <span class="text-base opacity-80">{new Date(wiki.updated_at * 1000).toLocaleString()}</span>
          </p>
        {/if}
      </div>
      <article class="prose w-full max-w-5xl p-6 pt-12">
        {#await contentRendered}
          <span class="loading loading-spinner loading-sm" />
          <span>{$i18n.t('wiki.rendering')}</span>
        {:then desc}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html desc}
        {/await}
      </article>
      <div class="h-32" />
  {/if}
</div>
