<script lang="ts">
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { admin } from '$lib/stores/admin'
  import type { Game, Notification } from '$lib/models/game'
  import { createNotification, deleteGameNotification, getGameNotifications } from '$lib/api/v1/game'
  import RxLink from '$lib/components/RxLink.svelte'
  import CreatePanel from './CreatePanel.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { blur } from 'svelte/transition'
  import { page } from '$app/stores'
  import { onDestroy } from 'svelte'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let loadingNotification = false
  let submitting = false
  let notifications: Notification[] = []
  let showCreatePanel = false
  let activeNotification: Notification = {
    id: 0,
    title: '',
    content: '',
    published_at: 0,
    game_id: 0,
  }

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--delete-20-regular]',
      label: '',
      level: 'error',
      type: 'button',
      onClick: (data: DTDataEntry) => {
        deleteModalOpened = true
        willDeletedNotification = {
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
      header: $i18n.t('notification.title'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    content: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    published_at: {
      header: $i18n.t('notification.publishedAt'),
      type: 'date',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    game_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  let deleteModalOpened = false
  let willDeletedNotification = {
    id: 0,
    title: '',
  }

  function fetchNotifications() {
    if (!$admin.game) return
    loading = true
    getGameNotifications($admin.game.id, currentPage, perPage)
      .then((res) => {
        notifications = res.notifications
        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('notification.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  let storedPage: number | undefined = undefined
  let storedGameId: number | undefined = undefined

  function watchPage(p: number, g: Game | null) {
    if (p && g && (p !== storedPage || storedGameId !== g.id)) {
      fetchNotifications()
      storedPage = p
      storedGameId = g.id
    }
  }

  $: watchPage(currentPage, $admin.game)

  function handleDeleteAnnouncement() {
    if (!$admin.game) return
    deleteGameNotification($admin.game.id, willDeletedNotification.id)
      .then(() => {
        showMessage('success', $i18n.t('notification.deleteSuccess'), 5000)
        deleteModalOpened = false
        fetchNotifications()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('notification.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      if (val.url.hash.replace('#', '') == 'create') {
        showCreatePanel = true
      }
    } else {
      showCreatePanel = false
      activeNotification = {
        id: 0,
        title: '',
        content: '',
        published_at: 0,
        game_id: 0,
      }
    }
  })

  onDestroy(() => {
    unsubscribe()
  })

  function CreateNotification() {
    if (!$admin.game) return
    submitting = true
    createNotification($admin.game.id, activeNotification)
      .then(() => {
        showMessage('success', $i18n.t('notification.createSuccess'), 5000)
        window.location.hash = ''
        fetchNotifications()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('notification.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        submitting = false
      })
  }
</script>

<svelte:head><title>{$i18n.t('admin.NotificationListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  {#if !showCreatePanel}
    <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
      <div class="h-16 flex flex-row items-center space-x-2">
        <h2 class="text-base font-bold flex-1">{$i18n.t('admin.NotificationListSettings')}</h2>
        <RxLink size="sm" level="info" href="#create">
          <span class="icon-[fluent--add-20-regular]"></span>
          <span>{$i18n.t('challenge.create')}</span>
        </RxLink>
      </div>
      <DataTable
        class="flex-1"
        {actions}
        data={notifications}
        {colDef}
        bind:page={currentPage}
        {total}
        {loading}
        booleanIconsDef={{
          hidden: {
            true: 'icon-[fluent--eye-off-20-regular] text-warning',
            false: 'icon-[fluent--checkmark-20-regular] text-success',
          },
        }}
      />
    </div>
  {:else}
    <CreatePanel
      class="flex-1"
      bind:notification={activeNotification}
      loading={loadingNotification}
      {submitting}
      on:close={() => {
        window.location.hash = ''
      }}
      on:submit={() => {
        CreateNotification()
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
        {$i18n.t('form.deleteConfirm', { item: decodeURI(willDeletedNotification.title.split('|')[0]) })}
      </h1>
      <div class="flex flex-row justify-end space-x-4">
        <RxButton size="sm" on:click={() => (deleteModalOpened = false)}>{$i18n.t('form.cancel')}</RxButton>
        <RxButton size="sm" level="error" on:click={handleDeleteAnnouncement}>{$i18n.t('form.confirm')}</RxButton>
      </div>
    </div>
  </div>
{/if}
