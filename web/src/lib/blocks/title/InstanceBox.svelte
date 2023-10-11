<script lang="ts">
  import { getChallenge } from '$lib/api/challenge'
  import RxButton from '$lib/components/RxButton.svelte'
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
    return Math.max(0, persistTime - duration)
  }

  let lastTime = calcLastTime($game.runningInstance)

  setInterval(() => {
    lastTime = calcLastTime($game.runningInstance)
  }, 1000)
</script>

<div class="p-2 flex flex-col">
  <div class="flex-1 flex flex-row items-center rounded-lg">
    <span class="icon-[fluent--engine-24-regular] w-12 h-12 text-success animate-pulse m-2 mx-4"></span>
    <div class="flex flex-col flex-1">
      <p class="text-base font-bold flex flex-row">
        {challenge?.name}
      </p>
      <p class="text-base font-bold opacity-60 flex flex-row">
        <span>T =&nbsp;</span>
        <span class="flex flex-row items-center space-x-0">
          <span>
            {Math.floor(lastTime / 3600)
              .toString()
              .padStart(2, '0')}
          </span>
          <span class="opacity-60">:</span>
          <span>
            {Math.floor((lastTime / 60) % 60)
              .toString()
              .padStart(2, '0')}
          </span>
          <span class="opacity-60">:</span>
          <span class="text-primary">
            {Math.floor(lastTime % 60)
              .toString()
              .padStart(2, '0')}
          </span>
        </span>
      </p>
    </div>
  </div>
  <div class="divider m-0 ml-2 mr-2" />
  <div class="flex flex-row space-x-2">
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--copy-16-regular] w-5 h-5 text-success"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--open-16-regular] w-5 h-5 text-success"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--timer-16-regular] w-5 h-5 text-info"></span>
    </RxButton>
    <RxButton ghost class="flex-1">
      <span class="icon-[fluent--circle-off-16-regular] w-5 h-5 text-error"></span>
    </RxButton>
  </div>
</div>
