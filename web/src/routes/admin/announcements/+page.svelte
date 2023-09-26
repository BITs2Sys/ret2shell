<script lang="ts">
  import { getAnnouncementList } from '$lib/api/announcement'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'

  let page: number = 1
  let perPage: number = 12
  let total: number = 0
  let loading = false
  let announcements: Announcement[] = []

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
      header: 'Title',
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    updated_at: {
      header: 'Updated At',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    published_at: {
      header: 'Created At',
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    publisher_id: {
      header: 'Publisher ID',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    content: {
      header: 'Content',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    pinned: {
      header: 'Pinned',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function fetchAnnouncements() {
    loading = true
    getAnnouncementList(page, perPage)
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
    if (page) {
      fetchAnnouncements()
    }
  }
</script>

<div class="flex-1 flex flex-col items-center">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.announcementsSettings')}</h2>
      <RxLink size="sm" level="info" href="/admin/announcements/create">
        <span class="icon-[fluent--add-16-regular] w-6 h-6" />
        <span class="text-base">{$i18n.t('action.create')}</span>
      </RxLink>
    </div>
    <DataTable
      class="flex-1"
      {actions}
      data={announcements}
      {colDef}
      bind:page
      {total}
      {loading}
      booleanIcon={{
        true: 'icon-[fluent--pin-16-regular] text-error',
        false: '',
      }}
    />
  </div>
</div>
