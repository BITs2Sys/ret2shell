<script lang="ts">
  import { getAnnouncement, getAnnouncementList } from '$lib/api/announcement'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import CreatePanel from './EditPanel.svelte'
  import { page } from '$app/stores'
  import { user } from '$lib/stores/user'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let announcements: Announcement[] = []
  let showCreatePanel = false

  let activeAnnouncement: Announcement = {
    id: 0,
    title: '',
    content: '',
    pinned: false,
    published_at: 0,
    publisher_id: 0,
    updated_at: 0,
  }

  $: renderedAnnouncements = announcements.map((a) => {
    return {
      ...a,
      title: `${encodeURI(a.title)}|${encodeURI(`#${a.id}`)}`,
    }
  })

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--edit-16-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
    },
    {
      icon: 'icon-[fluent--delete-16-regular]',
      label: '',
      level: 'error',
      type: 'button',
      onClick: (data: DTDataEntry) => {
        // popup delete modal
      },
    },
  ]
  let colDef: DTColumnsDef = {
    id: {
      header: 'ID',
      dimmed: true,
      type: 'number',
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    title: {
      header: $i18n.t('announcement.title'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    updated_at: {
      header: $i18n.t('announcement.updatedAt'),
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    published_at: {
      header: $i18n.t('announcement.publishedAt'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    publisher_id: {
      header: $i18n.t('announcement.author'),
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    content: {
      header: $i18n.t('announcement.content'),
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    pinned: {
      header: $i18n.t('announcement.pinned'),
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function fetchAnnouncements() {
    loading = true
    getAnnouncementList(currentPage, perPage)
      .then((res) => {
        announcements = res.announcements

        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('announcements.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  onMount(() => {
    fetchAnnouncements()
  })

  $: {
    if (currentPage) {
      fetchAnnouncements()
    }
  }
  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      const id = parseInt(val.url.hash.replace('#', ''))
      if (id && !Number.isNaN(id))
        getAnnouncement(id)
          .then((res) => {
            activeAnnouncement = res
            showCreatePanel = true
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('announcements.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
      else if (Number.isNaN(id)) {
        activeAnnouncement = {
          id: -1,
          title: '',
          content: '',
          pinned: false,
          published_at: 0,
          publisher_id: $user.id,
          updated_at: 0,
        }
        showCreatePanel = true
      }
    } else {
      showCreatePanel = false
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>

<div class="w-full flex-1 flex flex-col relative">
  <div class="h-16 flex flex-row items-center px-6 lg:px-12">
    <h2 class="text-base font-bold flex-1">{$i18n.t('admin.announcementsSettings')}</h2>
    <RxLink size="sm" level="info" href="#create">
      <span class="icon-[fluent--add-16-regular] w-6 h-6" />
      <span class="text-base">{$i18n.t('action.create')}</span>
    </RxLink>
  </div>
  <DataTable
    class="flex-1  px-6 lg:px-12"
    {actions}
    data={renderedAnnouncements}
    {colDef}
    bind:page={currentPage}
    {total}
    {loading}
    booleanIconsDef={{
      pinned: {
        true: 'icon-[fluent--pin-16-regular] text-error',
        false: '',
      },
    }}
  />
  <CreatePanel
    class={`transition-all ${showCreatePanel ? 'h-3/4' : 'h-0'}`}
    bind:announcement={activeAnnouncement}
    on:close={() => {
      window.location.hash = ''
    }}
  />
</div>
