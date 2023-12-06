<script lang="ts">
  import {
    createAnnouncement,
    deleteAnnouncement,
    getAnnouncement,
    getAnnouncementList,
    updateAnnouncement,
  } from '$lib/api/v1/announcement'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy } from 'svelte'
  import CreatePanel from './EditPanel.svelte'
  import { page } from '$app/stores'
  import { user } from '$lib/stores/user'
  import { platform } from '$lib/stores/platform'
  import RxButton from '$lib/components/RxButton.svelte'
  import { blur } from 'svelte/transition'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let loadingAnnouncement = false
  let submitting = false
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

  let deleteModalOpened = false
  let willDeletedAnnouncement = {
    id: 0,
    title: '',
  }

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--edit-20-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
    },
    {
      icon: 'icon-[fluent--delete-20-regular]',
      label: '',
      level: 'error',
      type: 'button',
      onClick: (data: DTDataEntry) => {
        deleteModalOpened = true
        willDeletedAnnouncement = {
          id: data.id as number,
          title: data.title as string,
        }
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

  $: {
    if (currentPage) {
      fetchAnnouncements()
    }
  }
  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      loadingAnnouncement = true
      const id = parseInt(val.url.hash.replace('#', ''))
      showCreatePanel = true
      if (id && !Number.isNaN(id)) {
        getAnnouncement(id)
          .then((res) => {
            activeAnnouncement = res
            showCreatePanel = true
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('announcements.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
          .finally(() => {
            loadingAnnouncement = false
          })
      } else if (Number.isNaN(id)) {
        activeAnnouncement = {
          id: -1,
          title: '',
          content: '',
          pinned: false,
          published_at: 0,
          publisher_id: $user.id,
          updated_at: 0,
        }
        loadingAnnouncement = false
      }
    } else {
      showCreatePanel = false
      activeAnnouncement = {
        id: -1,
        title: '',
        content: '',
        pinned: false,
        published_at: 0,
        publisher_id: $user.id,
        updated_at: 0,
      }
    }
  })

  onDestroy(() => {
    unsubscribe()
  })

  function updateOrCreateAnnouncement() {
    if (activeAnnouncement.title.length <= 0 || activeAnnouncement.content.length <= 0) {
      showMessage('error', $i18n.t('announcement.cantBeEmpty'), 5000)
      return
    }
    submitting = true
    if (activeAnnouncement.id > 0) {
      updateAnnouncement(activeAnnouncement.id, activeAnnouncement)
        .then(() => {
          showMessage('success', $i18n.t('announcement.updateSuccess'), 5000)
          window.location.hash = ''
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('announcement.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
    } else {
      const announcement: Announcement = {
        ...activeAnnouncement,
        id: 0,
        published_at: Math.floor(new Date().getTime() / 1000),
        publisher_id: $user.id,
      }
      createAnnouncement(announcement)
        .then(() => {
          showMessage('success', $i18n.t('announcement.createSuccess'), 5000)
          window.location.hash = ''
          fetchAnnouncements()
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('announcement.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
    }
  }

  function handleDeleteAnnouncement() {
    deleteAnnouncement(willDeletedAnnouncement.id)
      .then(() => {
        showMessage('success', $i18n.t('announcement.deleteSuccess'), 5000)
        deleteModalOpened = false
        fetchAnnouncements()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('announcement.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }
</script>

<svelte:head><title>{$i18n.t('admin.announcementsSettings')} - {$platform.name}</title></svelte:head>

<div class="w-full flex-1 flex flex-col relative">
  {#if !showCreatePanel}
    <div class="h-16 flex flex-row items-center px-6 lg:px-12 flex-shrink-0">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.announcementsSettings')}</h2>
      <RxLink size="sm" level="info" href="#create">
        <span class="icon-[fluent--add-20-regular]" />
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
          true: 'icon-[fluent--pin-20-regular] text-error',
          false: '',
        },
      }}
    />
  {:else}
    <CreatePanel
      class="flex-1"
      bind:announcement={activeAnnouncement}
      loading={loadingAnnouncement}
      {submitting}
      on:close={() => {
        window.location.hash = ''
      }}
      on:submit={() => {
        updateOrCreateAnnouncement()
      }}
    />
  {/if}
</div>
{#if deleteModalOpened}
  <div
    class="fixed top-0 left-0 w-full h-full bg-base-100/60 z-50 flex flex-col items-center justify-center"
    transition:blur={{ amount: 20, duration: 300 }}
  >
    <div class="rounded-box p-4 flex flex-col space-y-4 bg-neutral w-64">
      <h1 class="text-base font-bold">
        {$i18n.t('form.deleteConfirm', { item: decodeURI(willDeletedAnnouncement.title.split('|')[0]) })}
      </h1>
      <div class="flex flex-row justify-end space-x-4">
        <RxButton size="sm" on:click={() => (deleteModalOpened = false)}>{$i18n.t('form.cancel')}</RxButton>
        <RxButton size="sm" level="error" on:click={handleDeleteAnnouncement}>{$i18n.t('form.confirm')}</RxButton>
      </div>
    </div>
  </div>
{/if}
