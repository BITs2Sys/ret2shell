<script lang="ts">
  import { getChallenge } from '$lib/api/challenge'
  import Engine from '$lib/assets/animates/engine.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxTimer from '$lib/components/RxTimer.svelte'
  import type { Challenge } from '$lib/models/challenge'
  import type { Instance } from '$lib/models/instance'
  import { game } from '$lib/stores/game'
  import { onDestroy } from 'svelte'

  let challenge: Challenge | null = null

  let gameUnsubscribe = game.subscribe((value) => {
    if (value.runningInstance) {
      getChallenge(value.runningInstance.challenge_id).then((res) => {
        challenge = res
      })
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
  })

  function calcLastTime(instance: Instance | null) {
    if (instance === null) return 0
    const now = new Date().getTime()
    const start = instance.started_at * 1000
    const duration = (now - start) / 1000
    const persistTime = instance.renew_count * 3600
    let result = Math.max(0, persistTime - duration)
    if (result === 0) {
      $game.runningInstance = null
    }
    return result
  }

  let lastTime = calcLastTime($game.runningInstance)

  setInterval(() => {
    lastTime = calcLastTime($game.runningInstance)
  }, 1000)
</script>

<div class="p-2 flex flex-col">
  <div class="flex-1 flex flex-row items-center rounded-lg space-x-2">
    <div class="text-success m-2">
      <Engine width={48} height={48} />
    </div>
    <div class="flex flex-col flex-1">
      <p class="text-base font-bold flex flex-row">
        {challenge?.name}
      </p>
      <p class="text-base font-bold opacity-60 flex flex-row">
        <RxTimer time={lastTime} />
      </p>
    </div>
  </div>
  <div class="divider m-0 ml-2 mr-2" />
  <div class="flex flex-row space-x-2">
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--copy-20-regular] w-5 h-5 text-success"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--open-20-regular] w-5 h-5 text-success"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--timer-20-regular] w-5 h-5 text-info"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--circle-off-20-regular] w-5 h-5 text-error"></span>
    </RxButton>
  </div>
</div>
