<script lang="ts">
  import { game } from '$lib/stores/game'
  import RxImage from '$lib/components/RxImage.svelte'
  import { i18n } from '$lib/i18n'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxTag from '$lib/components/RxTag.svelte'
  import { canTakePartInGame } from '$lib/utils/auth'
  import { onDestroy } from 'svelte'
  import { getInstituteList } from '$lib/api/user'
  import type { Institute } from '$lib/models/institute'
  import RxTimer from '$lib/components/RxTimer.svelte'

  let isStarted = false
  let isEnded = false
  let unknownState = true
  $: {
    if ($game.current) {
      unknownState = false
      let startTime = new Date($game.current.start_time * 1000)
      let endTime = new Date($game.current.end_time * 1000)
      let now = new Date()
      isStarted = now >= startTime
      isEnded = now >= endTime
    } else {
      unknownState = true
    }
  }

  let timer = setInterval(() => {
    calcTime()
  }, 1000)
  let time = 0

  function calcTime() {
    let now = new Date()
    time = ($game.current?.end_time || 0) - now.getTime() / 1000
  }

  let canTakePartIn = false
  let institute: Institute | null = null

  let gameUnsubscribe = game.subscribe((value) => {
    if (value) {
      canTakePartInGame().then((res) => {
        canTakePartIn = res
      })
      if (value.current?.institute_id) {
        getInstituteList().then((res) => {
          institute = res.find((item) => item.id === value.current?.institute_id) || null
        })
      }
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
    clearInterval(timer)
  })
</script>

<svelte:head>
  <title>{$i18n.t('game.home')} - {$game.current?.name}</title>
</svelte:head>
<div class="h-48 bg-base-content/5 backdrop-blur relative">
  {#if $game.current?.cover_path}
    <RxImage class="absolute top-0 left-0 w-full h-full opacity-60" src={$game.current?.cover_path} loading={false}
    ></RxImage>
  {/if}
  <div class="absolute top-0 left-0 w-full h-full bg-base-100/60 flex flex-col justify-center items-center space-y-4">
    <h1 class="text-4xl font-bold">{$game.current?.name}</h1>
    <h2 class="text-3xl font-bold">
      {#if isStarted && !isEnded}
        <span class="flex flex-row items-center space-x-4">
          <span class="icon-[fluent--chevron-double-right-20-regular] opacity-40"></span>
          <RxTimer {time} />
          <span class="icon-[fluent--chevron-double-left-20-regular] opacity-40"></span>
        </span>
      {:else if isEnded}
        <span class="text-warning">{$i18n.t('games.ended')}</span>
      {:else if unknownState}
        <span class="text-error">{$i18n.t('games.unknown')}</span>
      {:else}
        <span class="text-info">{$i18n.t('games.notStarted')}</span>
      {/if}
    </h2>
  </div>
</div>
<div class="flex flex-row justify-center items-center p-6 space-x-2">
  <RxTag
    label={$game.current?.institute_id
      ? `${$i18n.t('games.restrictedGame')}: ${institute?.name}`
      : $i18n.t('games.publicGame')}
  />
  <RxTag
    label={($game.current?.team_size_limit || 0) > 1
      ? `${$i18n.t('games.multiPlayer', { limit: $game.current?.team_size_limit })}`
      : $i18n.t('games.singlePlayer')}
  />
  {#if $game.team}
    <RxTag label={`${$i18n.t('games.takePartInAs')}: ${$game.team.name}`} level="success" />
  {:else if canTakePartIn}
    <RxTag label={$i18n.t('games.canTakePartIn')} level="success" />
  {:else}
    <RxTag label={$i18n.t('games.cannotTakePartIn')} level="error" />
  {/if}
</div>
<RxArticle class="w-full self-center max-w-5xl p-6" content={$game.current?.introduction || ''} />
<div class="h-32"></div>
