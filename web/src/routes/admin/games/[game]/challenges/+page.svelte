<script lang="ts">
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { admin } from '$lib/stores/admin'
  import type { Game } from '$lib/models/game'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import RxButton from '$lib/components/RxButton.svelte'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let challenges: Challenge[] = []
  let tags: Tag[] = []
  let filterTagID: number | undefined = undefined

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
      header: $i18n.t('challenge.name'),
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

    game_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    tag_id: {
      header: $i18n.t('challenge.tag'),
      type: 'tag',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    hidden: {
      header: $i18n.t('challenge.status'),
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    initial_score: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    current_score: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    minimum_score: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    updated_at: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    decay: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    bucket: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    checker: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function fetchChallenges() {
    if (!$admin.game) return
    loading = true
    getChallengeList($admin.game.id, currentPage, perPage, filterTagID)
      .then((res) => {
        challenges = res.challenges
        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('challenge.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  function fetchTags() {
    loading = true
    getTagList()
      .then((res) => {
        tags = res
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('tag.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  $: renderChallenges = challenges.map((challenge) => {
    return {
      ...challenge,
      tag_id: tags.find((tag) => tag.id === challenge.tag_id)?.name,
    }
  })

  onMount(() => {
    fetchTags()
  })

  let storedPage: number | undefined = undefined
  let storedGameId: number | undefined = undefined

  function watchPage(p: number, g: Game | null) {
    if (p && g && (p !== storedPage || storedGameId !== g.id)) {
      fetchChallenges()
      storedPage = p
      storedGameId = g.id
    }
  }

  $: watchPage(currentPage, $admin.game)
  $: {
    if (filterTagID || !filterTagID) {
      fetchChallenges()
    }
  }
</script>

<svelte:head><title>{$i18n.t('admin.challengeListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.challengeListSettings')}</h2>
      <p class="text-base font-bold opacity-80">{$i18n.t('challenge.filterTag')}</p>
      <div class="relative w-64 flex flex-row">
        <RxSelect
          size="sm"
          name="tag_id"
          availableOptions={tags
            .map((i) => {
              return { id: i.id, label: i.name }
            }) //@ts-expect-error id is string | number | null
            .concat([{ id: null, label: 'NONE' }])}
          bind:value={filterTagID}
        />
      </div>
      <RxButton size="sm" level="primary">
        <span class="icon-[fluent--add-16-regular]"></span>
        <span>{$i18n.t('challenge.create')}</span>
      </RxButton>
    </div>
    <DataTable
      class="flex-1"
      {actions}
      data={renderChallenges}
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
