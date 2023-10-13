<script lang="ts">
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { admin } from '$lib/stores/admin'
  import type { Game, Notification } from '$lib/models/game'
  import { getGameNotifications } from '$lib/api/game'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let notifications: Notification[] = []

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--edit-16-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
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
</script>

<svelte:head><title>{$i18n.t('admin.NotificationListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.NotificationListSettings')}</h2>
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
          true: 'icon-[fluent--eye-off-16-regular] text-warning',
          false: 'icon-[fluent--checkmark-16-regular] text-success',
        },
      }}
    />
  </div>
</div>
