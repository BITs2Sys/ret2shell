<script lang="ts">
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { admin } from '$lib/stores/admin'
  import type { Game, Notification } from '$lib/models/game'
  import { getGameNotifications, getGameTeamList } from '$lib/api/game'
  import { State, type Team } from '$lib/models/team'
  import { getInstituteList } from '$lib/api/user'
  import { onMount } from 'svelte'
  import type { Institute } from '$lib/models/institute'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let teams: Team[] = []

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
    name: {
      header: $i18n.t('team.name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    game_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    token: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    state: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    institute_info: {
      header: $i18n.t('team.institute_info'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    score: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    history: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    last_active_at: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    hidden: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    banned: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    needAudit: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }
  let institutes: Institute[] = []
  $: renderTeams = teams.map((team) => {
    return {
      ...team,
      state: State[team.state],
      institute_info: institutes.find((i) => i.id === team.institute_id)?.name || '',
      needAudit: team.state === State.NeedAudit,
      banned: team.state === State.Banned,
      hidden: team.state === State.Hidden,
    }
  })

  function fetchTeams() {
    if (!$admin.game) return
    loading = true
    getGameTeamList($admin.game.id, currentPage, perPage)
      .then((res) => {
        teams = res.teams
        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('team.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }
  $: console.log(renderTeams)

  let storedPage: number | undefined = undefined
  let storedGameId: number | undefined = undefined

  function watchPage(p: number, g: Game | null) {
    if (p && g && (p !== storedPage || storedGameId !== g.id)) {
      fetchTeams()
      storedPage = p
      storedGameId = g.id
    }
  }

  $: watchPage(currentPage, $admin.game)

  function fetchInstitutes() {
    loading = true
    getInstituteList()
      .then((res) => {
        institutes = res
        console.log(institutes)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('institutes.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  onMount(() => {
    fetchInstitutes()
  })
</script>

<svelte:head><title>{$i18n.t('admin.teamListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.teamListSettings')}</h2>
    </div>
    <DataTable
      class="flex-1"
      {actions}
      data={renderTeams}
      {colDef}
      bind:page={currentPage}
      {total}
      {loading}
      booleanIconsDef={{
        hidden: {
          true: 'icon-[fluent--eye-off-16-regular] text-warning',
          false: '',
        },
        banned: {
          true: 'icon-[fluent--circle-off-16-regular] text-error',
          false: 'icon-[fluent--checkmark-circle-16-regular] text-success',
        },
        needAudit: {
          true: 'icon-[fluent--question-circle-16-regular] text-info',
          false: '',
        },
      }}
    />
  </div>
</div>
