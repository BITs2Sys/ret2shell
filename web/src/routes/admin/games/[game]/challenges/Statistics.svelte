<script lang="ts">
  import { getChallengeStatistics } from '$lib/api/challenge'
  import RxStatistics from '$lib/components/RxStatistics.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  export let challenge: Challenge
  let submissions_count: number = 0
  let solves_count: number = 0
  let instances_count: number = 0
  let running_instances_count: number = 0

  $: watchChallenge(challenge)
  function watchChallenge(chal: Challenge) {
    getChallengeStatistics(chal.id)
      .then((stats) => {
        submissions_count = stats.submissions_count
        solves_count = stats.solves_count
        instances_count = stats.instances_count
        running_instances_count = stats.running_instances_count
      })
      .catch((err) => {
        showMessage(
          'error',
          `${$i18n.t('challenge.fetchStatisticsFailed')}: ${(err as AxiosError).response?.data}`,
          5000
        )
      })
  }
</script>

<div class="w-full flex flex-1 flex-col">
  <div class="p-12 flex flex-row">
    <div class="flex flex-col flex-1 space-y-4">
      <div class="flex flex-row space-x-4">
        <RxStatistics
          icon="icon-[fluent--flag-24-filled] w-6 h-6 text-primary"
          title={$i18n.t('challenge.submissionCount')}
          value={submissions_count}
        ></RxStatistics>
        <RxStatistics
          icon="icon-[fluent--checkmark-circle-24-filled] w-6 h-6 text-success"
          title={$i18n.t('challenge.solvesCount')}
          value={solves_count}
        ></RxStatistics>
        <RxStatistics
          icon="icon-[fluent--data-pie-24-filled] w-6 h-6 text-primary"
          title={$i18n.t('challenge.solvesPercentage')}
          value={((solves_count / submissions_count) * 100).toFixed(0) + '%'}
        ></RxStatistics>
      </div>
      <div class="flex-1 flex flex-row space-x-4">
        <RxStatistics
          icon="icon-[fluent--number-symbol-24-filled] w-6 h-6 text-primary"
          title={$i18n.t('challenge.currentScore')}
          value={challenge.current_score}
        ></RxStatistics>
        <RxStatistics
          icon="icon-[fluent--engine-24-filled] w-6 h-6 text-primary"
          title={$i18n.t('challenge.instancesCount')}
          value={instances_count}
        ></RxStatistics>
        <RxStatistics
          icon="icon-[fluent--engine-24-filled] w-6 h-6 text-success"
          title={$i18n.t('challenge.runningInstancesCount')}
          value={running_instances_count}
        ></RxStatistics>
      </div>
    </div>
  </div>
</div>
