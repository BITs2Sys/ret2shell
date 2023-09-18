<script lang="ts">
  import { game } from '$lib/stores/game'
  import RxImage from '$lib/components/RxImage.svelte'
  import { i18n } from '$lib/i18n'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxTag from '$lib/components/RxTag.svelte'

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
  let hours = 0
  let minutes = 0
  let seconds = 0

  function calcTime() {
    let now = new Date()
    let diff = ($game.current?.end_time || 0) - now.getTime() / 1000
    if (diff <= 0) {
      clearInterval(timer)
      return
    }
    hours = Math.floor(diff / 3600)
    minutes = Math.floor((diff - hours * 3600) / 60)
    seconds = Math.floor(diff - hours * 3600 - minutes * 60)
  }

  $: canTakePartIn = true
</script>

<div class="h-48 bg-base-content/5 backdrop-blur relative">
  {#if $game.current?.cover_path}
    <RxImage
      class="absolute top-0 left-0 w-full h-full opacity-60"
      src={$game.current?.cover_path || ''}
      loading={false}
    ></RxImage>
  {/if}
  <div class="absolute top-0 left-0 w-full h-full bg-base-100/60 flex flex-col justify-center items-center space-y-4">
    <h1 class="text-4xl font-bold">{$game.current?.name}</h1>
    <h2 class="text-3xl font-bold">
      {#if isStarted && !isEnded}
        <span class="flex flex-row items-center space-x-0">
          <span class="icon-[fluent--chevron-double-right-16-regular] opacity-40 mr-4"></span>
          <span>{hours.toString().padStart(2, '0')}</span>
          <span class="opacity-60">:</span>
          <span>{minutes.toString().padStart(2, '0')}</span>
          <span class="opacity-60">:</span>
          <span class="text-primary">{seconds.toString().padStart(2, '0')}</span>
          <span class="icon-[fluent--chevron-double-left-16-regular] opacity-40 !ml-4"></span>
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
  <RxTag label={$game.current?.institute_id ? $i18n.t('games.restrictedGame') : $i18n.t('games.publicGame')} />
  <RxTag
    label={($game.current?.team_size_limit || 0) > 1
      ? `${$i18n.t('games.multiPlayer', { limit: $game.current?.team_size_limit })}`
      : $i18n.t('games.singlePlayer')}
  />
  {#if canTakePartIn}
    <RxTag label={$i18n.t('games.canTakePartIn')} level="success" />
  {:else}
    <RxTag label={$i18n.t('games.cannotTakePartIn')} level="error" />
  {/if}
</div>
<RxArticle class="w-full self-center max-w-5xl p-6" content={$game.current?.introduction || ''} />
<div class="h-32"></div>
