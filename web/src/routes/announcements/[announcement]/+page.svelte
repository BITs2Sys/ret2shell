<script lang="ts">
  import { page } from '$app/stores'
  import { getAnnouncement } from '$lib/api/announcement'
  import { getUserInfo } from '$lib/api/user'
  import Error from '$lib/blocks/Error.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import type { User } from '$lib/models/user'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import '$lib/styles/article.scss'
  let loading = true
  let error = 200
  let user: User | undefined = undefined
  let announcement: Announcement = {
    id: 0,
    title: $i18n.t('announcements.titleLoadingPlaceholder'),
    updated_at: 0,
    published_at: 0,
    publisher_id: 0,
    content: '',
    pinned: false,
  }
  const render = async (content: string) => {
    let { MarkTo } = await import('$lib/markto')
    let dompurify = await import('isomorphic-dompurify')
    const markTo = new MarkTo()
    await markTo.init({ type: 'html', options: { prism: true, katex: true } })
    return dompurify.sanitize((await markTo.render(content)) as string)
  }

  let contentRendered: Promise<string> = render(announcement.content)

  function scrollToView() {
    setTimeout(() => {
      document.getElementById(decodeURI(location.hash.replace('#', '')))?.scrollIntoView({ behavior: 'smooth' })
    }, 100)
  }
  onMount(() => {
    loading = true
    let id = parseInt($page.params['announcement']) || -1
    if (id < 0) {
      error = 404
      loading = false
      return
    }
    getAnnouncement(id)
      .then((data) => {
        announcement = data
        getUserInfo(data.publisher_id)
          .then((data) => {
            user = data
          })
          .catch(() => {
            user = undefined
          })
        contentRendered = render(announcement.content)
        contentRendered.then(() => {
          scrollToView()
        })
        loading = false
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('announcements.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        error = (err as AxiosError).response?.status || 500
      })
  })
</script>

<svelte:head><title>{announcement.title} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col self-center w-full p-4 lg:p-6 items-center">
  {#if loading}
    <div class="h-16 flex flex-row justify-center items-center space-x-2">
      <span class="loading loading-spinner loading-sm" />
      <span class="text-base">{$i18n.t('announcements.fetchingContent')}</span>
    </div>
  {:else if error - 200 > 100 && !loading}
    <Error status={error} />
  {:else}
    <h1 class="text-3xl font-bold h-16 mt-12 flex justify-center items-center">
      {announcement.title}
    </h1>
    <div class="flex flex-row space-x-4 flex-wrap justify-center">
      <p>
        <span class="text-base opacity-80">{$i18n.t('announcement.author')}</span>:
        <span class="text-base opacity-80">{user?.name || $i18n.t('announcement.unknownAuthor')}</span>
      </p>
      <p>
        <span class="text-base opacity-80">{$i18n.t('announcement.publishedAt')}</span>:
        <span class="text-base opacity-80">{new Date(announcement.published_at * 1000).toLocaleString()}</span>
      </p>
      {#if announcement.published_at !== announcement.updated_at}
        <p>
          <span class="text-base opacity-80">{$i18n.t('announcement.updatedAt')}</span>:
          <span class="text-base opacity-80">{new Date(announcement.updated_at * 1000).toLocaleString()}</span>
        </p>
      {/if}
    </div>
    <article class="prose max-w-5xl w-full p-6 pt-12">
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
