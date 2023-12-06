<script lang="ts">
  import type { DTColumnAction, DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import { createChallenge, getChallengeList, getTagList } from '$lib/api/v1/challenge'
  import { admin } from '$lib/stores/admin'
  import type { Game } from '$lib/models/game'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { page } from '$app/stores'
  import Error from '$lib/blocks/Error.svelte'
  import CreatePanel from './CreatePanel.svelte'
  import EditPanel from './EditPanel.svelte'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let challenges: Challenge[] = []
  let tags: Tag[] = []
  let filterTagID: number | null = null
  let submitting: boolean = false
  let loadingChallenge: boolean = false
  let activeChallenge: Challenge = {
    id: 0,
    name: '',
    content: '',
    game_id: 0,
    tag_id: 0,
    hidden: false,
    initial_score: 1000,
    current_score: 0,
    minimum_score: 500,
    updated_at: 0,
    decay: 10,
    bucket: '',
    checker: '',
  }

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--arrow-enter-left-20-regular]',
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
      header: '',
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

  $: renderedChallenges = challenges.map((challenge) => {
    return {
      ...challenge,
      name: `${encodeURI(challenge.name)}|#${encodeURI(challenge.id.toString())}`,
      tag_id: tags.find((tag) => tag.id === challenge.tag_id)?.name,
    }
  })

  onMount(() => {
    fetchTags()
  })

  let storedPage: number | undefined = undefined
  let storedGameId: number | undefined = undefined
  let storedFilter: number | null = null

  function watchPage(p: number, g: Game | null) {
    if (p && g && (p !== storedPage || storedGameId !== g.id)) {
      fetchChallenges()
      storedPage = p
      storedGameId = g.id
    }
  }

  $: watchPage(currentPage, $admin.game)
  $: {
    if (filterTagID !== storedFilter) {
      currentPage = 1
      fetchChallenges()
      storedFilter = filterTagID
    }
  }

  let showCreatePanel = false
  let showEditPanel = false

  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      const hash = val.url.hash.replace('#', '')
      if (hash == 'create') {
        showCreatePanel = true
      } else if (!isNaN(parseInt(hash))) {
        showEditPanel = true
      }
    } else {
      showCreatePanel = false
      showEditPanel = false
    }
  })

  onDestroy(() => {
    unsubscribe()
  })

  function handleCreateChallenge(newChallenge: Challenge) {
    if (!$admin.game) return
    submitting = true
    createChallenge($admin.game?.id, newChallenge)
      .then(() => {
        showMessage('success', $i18n.t('challenge.createSuccess'), 5000)
        window.location.hash = ''
        fetchChallenges()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('challenge.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        submitting = false
      })
  }
</script>

<svelte:head><title>{$i18n.t('admin.challengeListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  {#if !showCreatePanel && !showEditPanel}
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
        <RxLink size="sm" level="info" href="#create">
          <span class="icon-[fluent--add-20-regular]"></span>
          <span>{$i18n.t('challenge.create')}</span>
        </RxLink>
      </div>
      <DataTable
        class="flex-1"
        {actions}
        data={renderedChallenges}
        {colDef}
        bind:page={currentPage}
        {total}
        {loading}
        booleanIconsDef={{
          hidden: {
            true: 'icon-[fluent--eye-off-20-regular] text-warning',
            false: '',
          },
        }}
      />
    </div>
  {:else if showCreatePanel}
    <CreatePanel
      {tags}
      bind:challenge={activeChallenge}
      class="flex-1"
      loading={loadingChallenge}
      {submitting}
      on:close={() => {
        window.location.hash = ''
      }}
      on:submit={(event) => {
        handleCreateChallenge(event.detail)
      }}
    ></CreatePanel>
  {:else if showEditPanel}
    <EditPanel
      class="flex-1"
      on:close={() => {
        window.location.hash = ''
      }}
      {tags}
    ></EditPanel>
  {:else}
    <Error status={404} />
  {/if}
</div>
