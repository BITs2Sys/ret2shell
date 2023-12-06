<script lang="ts">
  import type { Challenge } from '$lib/models/challenge'
  import type { DTColumnsDef } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import { i18n } from '$lib/i18n'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { admin } from '$lib/stores/admin'
  import { onDestroy } from 'svelte'
  import type { SubmissionWithInfo } from '$lib/models/submission'
  import { getChallengeSubmission } from '$lib/api/v1/challenge'

  export let challenge: Challenge
  let currentPage: number = 1
  let perPage: number = 15
  let total: number = 0
  let loading = false
  let submissions: SubmissionWithInfo[] = []
  $: readerSubmissions = submissions.map((a) => {
    return {
      ...a,
      user_name: `${encodeURI(a.user_name)}|${encodeURI(`/admin/users/persons#${a.user_id.toString()}`)}`,
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
    created_at: {
      header: $i18n.t('submission.created_at'),
      dimmed: true,
      type: 'date',
      sizePolicy: 'shrink',
      justify: 'text-start',
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
      header: '',
      type: 'hidden',
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
    solved: {
      header: '',
      type: 'bool',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function refreshSubmissions() {
    loading = true
    getChallengeSubmission(challenge.id, currentPage, perPage)
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

<div class="lg:p-6 w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
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
