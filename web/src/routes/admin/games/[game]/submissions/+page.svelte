<script lang="ts">
  import type { DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { admin } from '$lib/stores/admin'
  import { getGameSubmission } from '$lib/api/v1/game'
  import { onDestroy } from 'svelte'
  import type { SubmissionWithInfo } from '$lib/models/submission'

  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let submissions: SubmissionWithInfo[] = []
  $: readerSubmissions = submissions.map((a) => {
    return {
      ...a,
      user_name: `${encodeURI(a.user_name)}|${encodeURI(`/admin/users/persons#${a.user_id.toString()}`)}`,
      challenge_name: `${encodeURI(a.challenge_name)}|${encodeURI(
        `/admin/games/${$admin.game?.id}/challenges#${a.challenge_id.toString()}`
      )}`,
    }
  })

  let colDef: DTColumnsDef = {
    id: {
      header: 'ID',
      dimmed: true,
      type: 'number',
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    user_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    user_name: {
      header: $i18n.t('submission.user_name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    challenge_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    challenge_name: {
      header: $i18n.t('submission.challenge_name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    tag_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    tag_name: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    content: {
      header: $i18n.t('submission.content'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    created_at: {
      header: $i18n.t('submission.created_at'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-start',
    },
    solved: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function refreshSubmissions() {
    if (!$admin.game) return
    loading = true
    getGameSubmission($admin.game.id, currentPage, perPage)
      .then((res) => {
        submissions = res.submissions
        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('submissions.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }
  const timer = setInterval(() => {
    refreshSubmissions()
  }, 5000)

  $: {
    if (currentPage && $admin.game) {
      refreshSubmissions()
    }
  }
  onDestroy(() => {
    clearInterval(timer)
  })
</script>

<svelte:head><title>{$i18n.t('admin.SubmissionListSettings')} - {$platform.name}</title></svelte:head>
<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.SubmissionListSettings')}</h2>
    </div>
    <DataTable
      class="flex-1"
      data={readerSubmissions}
      {colDef}
      bind:page={currentPage}
      {total}
      {loading}
      booleanIconsDef={{
        solved: {
          true: 'icon-[fluent--checkmark-circle-20-regular] text-success',
          false: 'icon-[fluent--dismiss-circle-20-regular] text-error',
        },
      }}
    />
  </div>
</div>
