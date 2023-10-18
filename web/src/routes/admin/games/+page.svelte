<script lang="ts">
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { platform } from '$lib/stores/platform'
  import { getGameList } from '$lib/api/game'
  import type { Game } from '$lib/models/game'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading: boolean = false
  let games: Game[] = []

  $: renderedGames = games.map((a) => {
    return {
      ...a,
      name: `${encodeURI(a.name)}|/admin/games/${encodeURI(a.id.toString())}`,
    }
  })

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--chevron-double-right-20-regular]',
      label: $i18n.t('admin.enter'),
      level: 'info',
      type: 'link',
      href: '/admin/games/{id}',
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
    updated_at: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    name: {
      header: $i18n.t('calendar.name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    brief: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    introduction: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    start_time: {
      header: $i18n.t('calendar.startTime'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    end_time: {
      header: $i18n.t('calendar.endTime'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    register_time: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    archive_time: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    hidden: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    frozen: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    host_as_game: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    team_size_limit: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    cover_path: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    enable_team_audit: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    can_register_after_started: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    institute_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
  }

  function fetchgames() {
    loading = true
    getGameList(currentPage, perPage)
      .then((res) => {
        games = res.games
        total = res.total
        //   console.log('games', games)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('game.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  $: {
    if (currentPage) {
      fetchgames()
    }
  }
</script>

<svelte:head><title>{$i18n.t('admin.gamesSettings')} - {$platform.name}</title></svelte:head>

<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.gamesSettings')}</h2>
      <RxLink size="sm" level="info" href="#create">
        <span class="icon-[fluent--add-20-regular]" />
        <span class="text-base">{$i18n.t('action.create')}</span>
      </RxLink>
    </div>
    <DataTable
      class="flex-1"
      {actions}
      data={renderedGames}
      {colDef}
      bind:page={currentPage}
      {total}
      {loading}
      booleanIconsDef={{
        hidden: {
          true: 'icon-[fluent--eye-off-20-regular] text-warning',
          false: '',
        },
        frozen: {
          true: 'icon-[fluent--weather-snowflake-20-regular] text-info',
          false: '',
        },
        host_as_game: {
          true: 'icon-[fluent--flag-20-regular] text-error',
          false: 'icon-[fluent--beaker-20-regular] text-info',
        },
      }}
    />
  </div>
</div>
