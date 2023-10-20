<script lang="ts">
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxTimer from '$lib/components/RxTimer.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import type { Instance } from '$lib/models/instance'
  import { game } from '$lib/stores/game'

  export let challenge: Challenge | null
  export let solved: boolean = false

  // user experience
  let challengeScrollExpanded = true
  let challengeEnvExpanded = false
  let challengeAttachmentExpanded = false

  function calcLastTime(instance: Instance | null) {
    if (instance === null) return 0
    const now = new Date().getTime()
    const start = instance.started_at * 1000
    const duration = (now - start) / 1000
    const persistTime = instance.renew_count * 3600
    return Math.max(0, persistTime - duration)
  }

  let lastTime = 0
  setInterval(() => {
    lastTime = calcLastTime($game.runningInstance)
  }, 1000)

  $: lastProgress = lastTime / (($game.runningInstance?.renew_count || 1) * 3600)
</script>

<div
  on:wheel|passive={(e) => {
    if (e.deltaY > 0) {
      challengeScrollExpanded = false
      challengeAttachmentExpanded = false
      challengeEnvExpanded = false
    } else {
      challengeScrollExpanded = true
    }
  }}
  class={`w-full transition-all ${
    challengeScrollExpanded ? 'h-32' : 'h-16'
  } backdrop-blur border-b border-b-base-content/10 flex flex-row justify-center sticky top-0`}
>
  <div class="w-full max-w-5xl flex flex-row px-6 items-center">
    <span
      class={`${
        solved
          ? 'icon-[fluent--checkmark-circle-24-regular] text-success'
          : 'icon-[fluent--question-circle-24-regular] text-base-content/60'
      } transition-all transform ${challengeScrollExpanded ? 'w-16 h-16 mr-4' : 'w-6 h-6 mr-2'}`}
    />
    <div class="flex flex-col">
      <h1
        class={`font-bold transition-all transform ${
          challengeScrollExpanded ? 'text-2xl' : 'text-base'
        } flex flex-row space-x-2 items-center`}
      >
        <span>{challenge?.name}</span>
      </h1>
      <p
        class={`overflow-hidden transition-all flex flex-col justify-end ${
          challengeScrollExpanded ? 'h-8 opacity-60' : 'h-0 opacity-0'
        }`}
      >
        {$i18n.t('playground.currentScore')}: {challenge?.current_score}&nbsp;
        {$i18n.t('playground.lastUpdatedAt')}: {new Date((challenge?.updated_at || 0) * 1000).toLocaleString()}
      </p>
    </div>
    <div class="flex-1"></div>
    <div class={`flex ${challengeScrollExpanded ? 'flex-col' : 'flex-row space-x-2'}`}>
      <RxButton
        ghost
        square={!challengeScrollExpanded}
        justify={challengeScrollExpanded ? 'start' : 'center'}
        on:click={() => {
          challengeAttachmentExpanded = !challengeAttachmentExpanded
          challengeEnvExpanded = false
          challengeScrollExpanded = true
        }}
      >
        <span class="icon-[fluent--archive-20-regular] w-5 h-5 text-warning"></span>
        {#if challengeScrollExpanded}
          <span class="hidden md:inline-block">{$i18n.t('playground.manageAttachments')}</span>
        {/if}
      </RxButton>
      <RxButton
        ghost
        square={!challengeScrollExpanded}
        justify={challengeScrollExpanded ? 'start' : 'center'}
        on:click={() => {
          challengeEnvExpanded = !challengeEnvExpanded
          challengeAttachmentExpanded = false
          challengeScrollExpanded = true
        }}
      >
        <span class="icon-[fluent--engine-20-regular] w-5 h-5 text-success"></span>
        {#if challengeScrollExpanded}
          <span class="hidden md:inline-block">{$i18n.t('playground.manageEnv')}</span>
        {/if}
      </RxButton>
    </div>
  </div>
</div>
<div
  class={`w-full transition-all backdrop-blur border-b overflow-hidden flex flex-row justify-center sticky ${
    challengeEnvExpanded || challengeAttachmentExpanded ? 'h-16 border-b-base-content/5' : 'h-0 border-b-transparent'
  } ${challengeScrollExpanded ? 'top-32' : 'top-16'}`}
>
  {#if challengeAttachmentExpanded}
    <div class="w-full max-w-5xl flex flex-row items-center px-6 space-x-2">
      <span class="font-bold text-base opacity-60">{$i18n.t('playground.attachmentCount')}:</span>
      <span class="font-bold text-base">{0}</span>
      <div class="flex-1"></div>
      <span class="font-bold text-base opacity-60 hidden lg:inline-block">{$i18n.t('playground.quickAction')}:</span>
      <RxButton ghost>
        <span class="icon-[fluent--apps-list-20-regular] w-5 h-5"></span>
        <span class="hidden md:inline-block">{$i18n.t('playground.listAllAttachment')}</span>
      </RxButton>
      <RxButton ghost>
        <span class="icon-[fluent--cloud-arrow-down-20-regular] w-5 h-5"></span>
        <span class="hidden md:inline-block">{$i18n.t('playground.packAndDownload')}</span>
      </RxButton>
    </div>
  {:else if challengeEnvExpanded}
    <div class="w-full max-w-5xl flex flex-col items-center justify-center px-6 space-x-2">
      <div class="join w-full items-center">
        <RxButton ghost class="flex-1 join-item relative">
          {#if $game.runningInstance && challenge}
            <div class="absolute h-[2px] bottom-1 left-4 right-4 bg-neutral">
              <div
                class="h-[2px] opacity-40"
                class:bg-success={lastProgress > 0.6}
                class:bg-warning={lastProgress <= 0.6 && lastProgress > 0.2}
                class:bg-error={lastProgress <= 0.2}
                style={`width: ${lastProgress * 100}%`}
              ></div>
            </div>
          {/if}
          <span class="icon-[fluent--flow-20-regular] w-5 h-5"></span>
          {#if $game.runningInstance && challenge && $game.runningInstance.challenge_id === challenge.id}
            <span class="flex-1 text-left text-ellipsis overflow-hidden whitespace-nowrap">
              {$game.runningInstance.wsrx}
            </span>
          {:else if $game.runningInstance && challenge}
            <span class="flex-1 text-left text-warning opacity-60 text-ellipsis overflow-hidden whitespace-nowrap">
              {$i18n.t('playground.currentEnvNotMatch')}
            </span>
          {:else}
            <span class="flex-1 text-left opacity-60 text-ellipsis overflow-hidden whitespace-nowrap">
              {$i18n.t('playground.noRunningEnv')}
            </span>
          {/if}
          {#if $game.runningInstance && challenge && $game.runningInstance.challenge_id === challenge.id}
            <span class="opacity-60">{$i18n.t('playground.envLastTime')}</span>
            <RxTimer time={lastTime} />
            <RxButton ghost square class="join-item ml-0">
              <span class="icon-[fluent--copy-20-regular] w-5 h-5 text-success"></span>
            </RxButton>
            <RxButton ghost square class="join-item ml-0">
              <span class="icon-[fluent--open-20-regular] w-5 h-5 text-info"></span>
            </RxButton>
          {/if}
        </RxButton>
        {#if $game.runningInstance && challenge && $game.runningInstance.challenge_id === challenge.id}
          <div class="w-[2px] h-[2rem] bg-base-content/20 mx-2"></div>
          <RxButton ghost class="join-item ml-0" square>
            <span class="icon-[fluent--timer-20-regular] w-5 h-5 text-info"></span>
          </RxButton>
          <RxButton ghost class="join-item ml-0" square>
            <span class="icon-[fluent--circle-off-20-regular] w-5 h-5 text-error"></span>
          </RxButton>
        {:else if $game.runningInstance && challenge}
          <RxButton ghost class="join-item ml-0" square>
            <span class="icon-[fluent--circle-off-20-regular] w-5 h-5 text-error"></span>
          </RxButton>
        {:else}
          <RxButton ghost class="join-item ml-0 text-success">
            <span class="icon-[fluent--play-20-regular] w-5 h-5"></span>
            <span>{$i18n.t('playground.launch')}</span>
          </RxButton>
        {/if}
      </div>
    </div>
  {/if}
</div>
<div
  class="flex flex-col w-full px-6 flex-1"
  on:wheel|passive={(e) => {
    if (e.deltaY > 0) {
      challengeScrollExpanded = false
      challengeAttachmentExpanded = false
      challengeEnvExpanded = false
    } else {
      challengeScrollExpanded = true
    }
  }}
>
  <RxArticle
    class="mt-12 w-full max-w-5xl self-center px-6"
    content={challenge?.content || $i18n.t('playground.emptyContent')}
  />
  <div class="h-12" />
</div>
