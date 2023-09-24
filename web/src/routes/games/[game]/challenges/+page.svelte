<script lang="ts">
  import { goto } from '$app/navigation'
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameNotifications, getGameSelfSubmission } from '$lib/api/game'
  import GameChallengeSidebar from '$lib/blocks/GameChallengeSidebar.svelte'
  import GameTeamSidebar from '$lib/blocks/GameTeamSidebar.svelte'
  import HintsPanel from '$lib/blocks/HintsPanel.svelte'
  import TerminalPanel from '$lib/blocks/TerminalPanel.svelte'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Submission } from '$lib/models/submission'
  import { Permission } from '$lib/models/user'
  import { game } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onDestroy, onMount } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { fly, blur } from 'svelte/transition'
  import { page } from '$app/stores'
  import { getChallenge } from '$lib/api/challenge'
  import Split from 'split.js'
  import type { Notification } from '$lib/models/game'

  let screenWidth: number
  let toggleSidebar = false
  let toggleTeamSidebar = false
  $: showSidebar = screenWidth > 1024 // lg
  $: showTeamSidebar = screenWidth > 1280 // lg

  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  let selfSubmissions: Submission[] = []
  let challenges: Challenge[] = []
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  let tags: Tag[] = []
  let currentGameIdCache: number | null = null

  function getChallenges() {
    if ($game.current?.id) {
      getChallengeList($game.current?.id, challengePage, challengePageSize)
        .then((res) => {
          challenges = challenges.concat(res.challenges)
          $game.challenges = challenges
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.noMoreChallenges')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  function getMoreChallenges() {
    challengePage++
    if (challengePage <= challengeTotalPages) getChallenges()
  }

  function getSelfSubmissions() {
    if ($game.current?.id) {
      getGameSelfSubmission($game.current?.id)
        .then((res) => {
          selfSubmissions = res
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchSelfSubmissionsFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
    }
  }

  onMount(() => {
    if (!$game.team && !$user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)) {
      goto(`/games/${$game.current?.id}`).then(() => {
        showMessage('warning', $i18n.t('games.takePartInFirst'), 5000)
      })
    } else if ($user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)) {
      // showMessage('info', $i18n.t('games.adminWarning'), 5000)
    }
  })

  let gameUnsubscribe = game.subscribe((value) => {
    if (value.current?.id && currentGameIdCache !== value.current.id) {
      currentGameIdCache = value.current.id
      challengePage = 1
      challenges = []
      getChallenges()
      getSelfSubmissions()
      getTagList()
        .then((res) => {
          tags = res
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
    $game.challenges = []
  })

  onMount(() => {
    Split(['#info-stack', '#work-stack'], {
      direction: 'vertical',
      gutterSize: 8,
      gutterAlign: 'center',
      minSize: 200,
      sizes: [70, 30],
      gutterStyle: (_dimension, gutterSize) => {
        return {
          height: `${gutterSize}px`,
          cursor: 'row-resize',
        }
      },
      gutter: (_index, direction) => {
        const gutter = document.createElement('div')
        gutter.className = `gutter gutter-${direction} border-b border-b-base-content/10 hover:border-b-primary transition-all duration-300`
        return gutter
      },
    })
  })
  // challenge reactive
  let openedChallenges: Challenge[] = []
  let activeChallenge: Challenge | null = null
  let loadingNewChallenge = false
  let loadingPlaceHolder: HTMLDivElement
  let openedTabDivRecord: Record<number, HTMLDivElement> = {}

  let unsubscribe = page.subscribe((value) => {
    let challengeId = value.url.hash ? parseInt(value.url.hash.slice(1)) || null : null
    if (challengeId) {
      if (openedChallenges.find((chal) => chal.id === challengeId)) {
        activeChallenge = openedChallenges.find((chal) => chal.id === challengeId) || null
        if (openedTabDivRecord[challengeId]) {
          setTimeout(() => {
            if (challengeId && openedTabDivRecord[challengeId])
              openedTabDivRecord[challengeId].scrollIntoView({ behavior: 'smooth', inline: 'center' })
          }, 0)
        }
        return
      }
      loadingNewChallenge = true
      setTimeout(() => {
        if (loadingPlaceHolder) loadingPlaceHolder.scrollIntoView({ behavior: 'smooth' })
      }, 0)
      getChallenge(challengeId)
        .then((value) => {
          openedChallenges.push(value)
          challengeScrollExpanded = true
          activeChallenge = value
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchChallengeFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
        .finally(() => {
          loadingNewChallenge = false
        })
    } else {
      activeChallenge = null
    }
  })
  onDestroy(unsubscribe)

  // user experience
  let challengeScrollExpanded = true
  let challengeEnvExpanded = false
  let challengeAttachmentExpanded = false

  // bottom panel
  let bottomTab = 0

  // notifications
  let notifications: Notification[] = []

  function fetchNotifications() {
    if ($game.current) {
      getGameNotifications($game.current.id, 1, 20)
        .then((res) => {
          notifications = res.notifications
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchNotificationsFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
    }
  }

  let timer = setInterval(() => {
    fetchNotifications()
  }, 5000)

  onDestroy(() => {
    clearInterval(timer)
  })
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row overflow-x-hidden">
  {#if showSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-r border-r-base-content/10 overflow-hidden"
    >
      <GameChallengeSidebar
        {selfSubmissions}
        {challenges}
        {tags}
        {mayHaveMoreChallenges}
        on:loadMoreChallenges={getMoreChallenges}
      />
    </div>
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed left-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleSidebar = !toggleSidebar
        }}
      />
      <span class="swap-off icon-[fluent--navigation-16-regular] fill-current w-6 h-6"></span>
      <span class="swap-on icon-[fluent--dismiss-16-regular] fill-current w-6 h-6"></span>
    </label>
  {/if}
  <div class="flex-1 flex flex-col overflow-x-hidden">
    <div id="info-stack" class="flex flex-col overflow-x-hidden">
      <div
        class="border-b border-b-base-content/10 flex flex-row items-center pr-2 space-x-2 backdrop-blur relative overflow-x-scroll flex-shrink-0 h-16 overflow-y-hidden"
        on:wheel={(e) => {
          e.currentTarget.scrollLeft += e.deltaY
        }}
      >
        <div class="bg-base-100 sticky left-0 p-2 flex-shrink-0 z-20">
          <RxLink ghost active={activeChallenge === null} href="#">
            <span class="w-4 h-4 icon-[fluent--pin-16-regular]" />
            {$i18n.t('playground.gameIntro')}
          </RxLink>
        </div>
        {#each openedChallenges as chal}
          <div
            class="join flex-shrink-0 transition-all"
            transition:fly={{
              x: -100,
              duration: 300,
              delay: 0,
              easing: quintOut,
            }}
            bind:this={openedTabDivRecord[chal.id]}
          >
            <RxLink
              class="join-item overflow-x-hidden max-w-[240px] flex-nowrap"
              ghost
              active={activeChallenge?.id === chal.id}
              href={`#${chal.id}`}
            >
              <span class="w-4 h-4 icon-[fluent--braces-16-regular] flex-shrink-0" />
              <span class="text-ellipsis overflow-hidden whitespace-nowrap">
                {chal.name}
              </span>
            </RxLink>
            <RxButton
              class="join-item ml-0"
              ghost
              on:click={() => {
                openedChallenges = openedChallenges.filter((c) => c.id !== chal.id)
                if (activeChallenge?.id === chal.id) {
                  activeChallenge = null
                  window.location.hash = '#'
                }
              }}
            >
              <span class="w-4 h-4 icon-[fluent--dismiss-16-regular]" />
            </RxButton>
          </div>
        {/each}
        {#if loadingNewChallenge}
          <div
            class="flex flex-row items-center space-x-2 opacity-80 w-48 flex-shrink-0"
            bind:this={loadingPlaceHolder}
          >
            <span class="loading loading-spinner loading-sm"></span>
            <span>{$i18n.t('playground.challengeLoading')}</span>
          </div>
        {/if}
      </div>
      <div class="flex-1 relative">
        <div class="absolute w-full h-full">
          <OverlayScrollbarsComponent
            options={{
              scrollbars: {
                theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light',
                autoHide: 'scroll',
              },
            }}
            class="relative w-full h-full print:h-auto print:overflow-auto"
            defer
          >
            <div
              class="w-full min-h-full flex flex-col items-center"
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
              {#if activeChallenge}
                <div
                  class={`w-full transition-all ${
                    challengeScrollExpanded ? 'h-32' : 'h-16'
                  } backdrop-blur bg-base-100/80 border-b border-b-base-content/5 flex flex-row justify-center sticky top-0`}
                >
                  <div class="w-full max-w-5xl flex flex-row px-6 items-center">
                    <span
                      class={`icon-[fluent--braces-16-regular] transition-all transform ${
                        challengeScrollExpanded ? 'w-12 h-12 mr-4 text-primary' : 'w-6 h-6 mr-2 text-error'
                      }`}
                    />
                    <div class="flex flex-col">
                      <h1
                        class={`font-bold transition-all transform ${
                          challengeScrollExpanded ? 'text-2xl' : 'text-base'
                        } flex flex-row space-x-2 items-center`}
                      >
                        <span>{activeChallenge.name}</span>
                      </h1>
                      <p
                        class={`overflow-hidden transition-all flex flex-col justify-end ${
                          challengeScrollExpanded ? 'h-8 opacity-60' : 'h-0 opacity-0'
                        }`}
                      >
                        {$i18n.t('playground.currentScore')}: {activeChallenge.current_score}&nbsp;
                        {$i18n.t('playground.lastUpdatedAt')}: {new Date(
                          activeChallenge.updated_at * 1000
                        ).toLocaleString()}
                      </p>
                    </div>
                    <div class="flex-1"></div>
                    <div class={`flex ${challengeScrollExpanded ? 'flex-col' : 'flex-row space-x-2'}`}>
                      <RxButton
                        ghost
                        square={!challengeScrollExpanded}
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
                    challengeEnvExpanded || challengeAttachmentExpanded
                      ? 'h-16 border-b-base-content/5'
                      : 'h-0 border-b-transparent'
                  } ${challengeScrollExpanded ? 'top-32' : 'top-16'}`}
                >
                  {#if challengeAttachmentExpanded}
                    <div class="w-full max-w-5xl flex flex-row items-center px-6 space-x-2">
                      <span class="font-bold text-base opacity-60">{$i18n.t('playground.attachmentCount')}:</span>
                      <span class="font-bold text-base">{0}</span>
                      <div class="flex-1"></div>
                      <span class="font-bold text-base opacity-60 hidden lg:inline-block"
                        >{$i18n.t('playground.quickAction')}:</span
                      >
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
                      <span class="font-bold text-base opacity-60 hidden md:inline-block"
                        >{$i18n.t('playground.quickAction')}:</span
                      >
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
                <div class="flex flex-col w-full max-w-5xl px-6">
                  <RxArticle class="mt-12" content={activeChallenge?.content || $i18n.t('playground.emptyContent')} />
                  <div class="h-12" />
                </div>
              {:else}
                <div class="flex flex-col w-full max-w-5xl px-6">
                  <h1 class="font-bold text-center text-3xl p-6 pt-12 pb-0">
                    {$game.current?.name}
                  </h1>
                  <RxArticle class="mt-4" content={$game.current?.introduction || $i18n.t('playground.emptyContent')} />
                  <div class="h-32" />
                </div>
              {/if}
            </div>
          </OverlayScrollbarsComponent>
        </div>
        {#if loadingNewChallenge}
          <div
            class="absolute bg-base-100/60 w-full h-full backdrop-blur-xl flex justify-center items-center space-x-2"
            transition:blur={{ amount: 20, duration: 300 }}
          >
            <span class="loading loading-sm loading-spinner"></span>
            <span>{$i18n.t('playground.challengeLoading')}</span>
          </div>
        {/if}
      </div>
    </div>
    <div id="work-stack" class="flex flex-col backdrop-blur">
      <div class="border-b border-b-base-content/5 flex flex-row items-center p-2 space-x-2">
        <RxButton
          ghost
          active={bottomTab === 0}
          on:click={() => {
            bottomTab = 0
          }}
        >
          <span class="w-5 h-5 icon-[fluent--code-16-regular]" />
          {$i18n.t('playground.terminal')}
        </RxButton>
        <RxButton
          ghost
          active={bottomTab === 1}
          on:click={() => {
            bottomTab = 1
          }}
        >
          <span class="w-5 h-5 icon-[fluent--info-16-regular]" />
          {$i18n.t('playground.challengeHints')}
        </RxButton>
      </div>
      <TerminalPanel game={$game.current} challenge={activeChallenge} availableChallenges={$game.challenges} class={bottomTab === 0 ? 'p-6' : 'hidden'} />
      <HintsPanel class={bottomTab === 1 ? '' : 'hidden'} />
    </div>
  </div>
  {#if showTeamSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-l border-l-base-content/10 overflow-hidden"
    >
      <GameTeamSidebar {notifications} />
    </div>
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleTeamSidebar = !toggleTeamSidebar
        }}
      />
      <span class="swap-off icon-[fluent--flag-16-regular] fill-current w-6 h-6"></span>
      <span class="swap-on icon-[fluent--dismiss-16-regular] fill-current w-6 h-6"></span>
    </label>
  {/if}
  {#if toggleTeamSidebar && !showTeamSidebar}
    <div
      class="fixed right-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-l border-l-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: 256, y: 0, opacity: 0, easing: quintOut }}
    >
      <GameTeamSidebar {notifications} />
    </div>
  {/if}
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed left-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <GameChallengeSidebar
        {selfSubmissions}
        {challenges}
        {tags}
        {mayHaveMoreChallenges}
        on:loadMoreChallenges={getMoreChallenges}
      />
    </div>
  {/if}
</div>
