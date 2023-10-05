<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import { fly } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'
  import { onMount } from 'svelte'
  import { getUserInfo, getUserTeams } from '$lib/api/user'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import type { TeamWithGameName } from '$lib/models/team'

  let toggleSidebar = false
  let screenWidth: number
  let loading = false
  let error = 200
  let teams: TeamWithGameName[] = []
  $: showSidebar = screenWidth > 1024 // lg

  onMount(() => {
    loading = true
    getUserInfo($user.id)
      .then((value) => {
        user.update((val) => {
          val = {
            ...val,
            info: value,
          }
          return val
        })
        loading = false
      })
      .catch((err) => {
        error = (err as AxiosError).response?.status || 500
      })
    getUserTeams($user.id)
      .then((value) => {
        teams = value
      })
      .catch((err) => {
        error = (err as AxiosError).response?.status || 500
      })
  })
</script>

<svelte:head><title>{$i18n.t('account.profile')} - {$platform.name}</title></svelte:head>
<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row">
  {#if showSidebar}
    <div
      class="fixed w-1/5 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-r border-r-base-content/10 print:hidden"
    >
      <Sidebar {loading} />
    </div>
    <div class="w-1/5 min-w-[24rem] max-w-[32rem] flex-shrink-0 print:hidden" />
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
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
  {#if error - 200 < 100}
    <div class="flex-1 flex flex-col items-center p-4 lg:p-6">
      <div class="w-full max-w-5xl flex flex-col">
        <h2 class="h-12 text-base font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10">
          <span class="icon-[fluent--notepad-20-regular] w-5 h-5"></span>
          <span>{$i18n.t('account.intro')}</span>
        </h2>
        <RxArticle class="mt-4" content={$user.info?.intro || $i18n.t('account.noIntro')}></RxArticle>
        <h2
          class="h-12 text-base font-bold flex flex-row space-x-2 items-center mt-4 border-b border-b-base-content/10"
        >
          <span class="icon-[fluent--data-bar-vertical-ascending-16-regular] w-5 h-5"></span>
          <span>{$i18n.t('account.recentActivities')}</span>
        </h2>
        <p class="flex flex-col space-y-2">
          {#each teams as team}
            <div class="h-12 flex flex-row items-center space-x-2 border-b border-b-base-content/5 mt-2">
              <span class="icon-[fluent--trophy-20-regular] w-5 h-5"></span>
              <span class="text-base flex-1"
                >{$i18n.t('account.takePartAs', { team: team.name, game: team.game_name, score: team.score })}</span
              >
              <span class="text-base opacity-60 px-4">
                {new Date(team.last_active_at * 1000).toLocaleDateString('default', {
                  year: 'numeric',
                  day: '2-digit',
                  month: '2-digit',
                })}
              </span>
            </div>
          {/each}
        </p>
      </div>
    </div>
  {:else}
    <Error status={error} />
  {/if}
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10 print:hidden"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <Sidebar {loading} />
    </div>
  {/if}
</div>
