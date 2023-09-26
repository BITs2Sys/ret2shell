<script lang="ts">
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'

  export let challenge: Challenge | null
  export let solved: boolean = false

  // user experience
  let challengeScrollExpanded = true
  let challengeEnvExpanded = false
  let challengeAttachmentExpanded = false
</script>

<div
  on:wheel={(e) => {
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
  } backdrop-blur bg-base-100/80 border-b border-b-base-content/5 flex flex-row justify-center sticky top-0`}
>
  <div class="w-full max-w-5xl flex flex-row px-6 items-center">
    <span
      class={`icon-[fluent--braces-16-regular] transition-all transform text-primary ${
        challengeScrollExpanded ? 'w-12 h-12 mr-4' : 'w-6 h-6 mr-2'
      }`}
    />
    <div class="flex flex-col">
      <h1
        class={`font-bold transition-all transform ${
          challengeScrollExpanded ? 'text-2xl' : 'text-base'
        } flex flex-row space-x-2 items-center`}
      >
        {#if solved}
          <span class="text-success">[{$i18n.t('challenges.solved')}]</span>
        {:else}
          <span class="text-warning">[{$i18n.t('challenges.unsolved')}]</span>
        {/if}
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
        <span class="icon-[fluent--archive-16-regular] w-6 h-6 text-warning"></span>
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
        <span class="icon-[fluent--engine-20-regular] w-6 h-6 text-success"></span>
        {#if challengeScrollExpanded}
          <span class="hidden md:inline-block">{$i18n.t('playground.manageEnv')}</span>
        {/if}
      </RxButton>
    </div>
  </div>
</div>
<div
  class={`w-full transition-all bg-base-100/80 backdrop-blur border-b overflow-hidden flex flex-row justify-center sticky ${
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
    <div class="w-full max-w-5xl flex flex-row items-center px-6 space-x-2">
      <div class="join">
        <RxButton ghost class="max-w-xs join-item">
          <span class="icon-[fluent--flow-16-regular] w-5 h-5"></span>
          <span class="flex-1 text-left opacity-60 text-ellipsis overflow-hidden whitespace-nowrap">
            {$i18n.t('playground.noRunningEnv')}
          </span>
        </RxButton>
        <RxButton ghost square class="join-item ml-0">
          <span class="icon-[fluent--copy-16-regular] w-5 h-5 text-success"></span>
        </RxButton>
        <RxButton ghost square class="join-item ml-0">
          <span class="icon-[fluent--open-16-regular] w-5 h-5 text-info"></span>
        </RxButton>
      </div>
      <span class="text-base font-bold opacity-60">{$i18n.t('playground.envLastTime')}:</span>
      <span class="text-base font-bold">--:--:--</span>
      <div class="flex-1"></div>
      <span class="font-bold text-base opacity-60 hidden md:inline-block">{$i18n.t('playground.quickAction')}:</span>
      <RxButton ghost square>
        <span class="icon-[fluent--play-16-regular] w-5 h-5 text-success"></span>
      </RxButton>
      <RxButton ghost square>
        <span class="icon-[fluent--timer-16-regular] w-5 h-5 text-info"></span>
      </RxButton>
      <RxButton ghost square>
        <span class="icon-[fluent--circle-off-16-regular] w-5 h-5 text-error"></span>
      </RxButton>
    </div>
  {/if}
</div>
<div
  class="flex flex-col w-full max-w-5xl px-6 flex-1"
  on:wheel={(e) => {
    if (e.deltaY > 0) {
      challengeScrollExpanded = false
      challengeAttachmentExpanded = false
      challengeEnvExpanded = false
    } else {
      challengeScrollExpanded = true
    }
  }}
>
  <RxArticle class="mt-12" content={challenge?.content || $i18n.t('playground.emptyContent')} />
  <div class="h-12" />
</div>
