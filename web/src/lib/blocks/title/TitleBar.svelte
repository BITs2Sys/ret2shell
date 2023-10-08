<script lang="ts">
  import logo from '$lib/assets/logo.svg'
  import { platform } from '$lib/stores/platform'
  import { user, userInfo } from '$lib/stores/user'
  import GlobalMenu from './GlobalMenu.svelte'
  import CustomizeBox from './CustomizeBox.svelte'
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxPopup from '$lib/components/RxPopup.svelte'
  import UserBox from './UserBox.svelte'
  import { initConfig } from '$lib/stores/init'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import { game } from '$lib/stores/game'
  import GameMenu from './GameMenu.svelte'
  import TeamBox from './TeamBox.svelte'
  import { canTakePartInGame } from '$lib/utils/auth'
  import { onDestroy, onMount } from 'svelte'
  import { Permission, type User } from '$lib/models/user'

  let canTakePartIn = false

  let gameUnsubscribe = game.subscribe((value) => {
    if (value) {
      canTakePartInGame().then((res) => {
        canTakePartIn = res
      })
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
  })

  let userFullInfo: User | null = null
  let loadingAvatar = false

  onMount(() => {
    loadingAvatar = true
    userInfo().then((value) => {
      userFullInfo = value
      loadingAvatar = false
    })
  })
</script>

<div id="page-top" />
<div
  class="navbar w-auto backdrop-blur shadow bg-neutral/80 transition-shadow z-50 print:hidden px-2 py-0 sticky top-0"
>
  {#if !$initConfig.processing}
    <RxPopup class="btn-square btn-ghost xl:hidden" name="navPopup" popupWidth={64}>
      <span slot="button" class="icon-[fluent--navigation-16-regular] w-5 h-5" />
      <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
        <ul class="menu menu-vertical">
          {#if $game.current}
            <GameMenu />
          {:else}
            <GlobalMenu />
          {/if}
        </ul>
      </div>
    </RxPopup>
  {/if}
  {#if $initConfig.processing}
    <ul class="menu menu-horizontal px-6 space-x-2 hidden xl:flex">
      <li>
        <RxButton ghost>
          <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
          <span>{$i18n.t('init.title')}</span>
          <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
        </RxButton>
      </li>
    </ul>
  {:else}
    <RxLink ghost href={$game.current && $game.showGameNav ? `/games/${$game.current.id}` : '/'} exactlyMatched>
      <img class="hidden xl:block" width="28" height="28" src={logo} alt="logo" />
      {#if $game.showGameNav}
        <span>{$game.current?.name}</span>
      {:else}
        <span>{$platform.name}</span>
      {/if}
    </RxLink>
    <ul class="menu menu-horizontal px-6 space-x-2 hidden xl:flex">
      {#if $game.showGameNav}
        <GameMenu />
      {:else}
        <GlobalMenu />
      {/if}
    </ul>
  {/if}
  <div class="flex-1" />
  <RxPopup class="btn-square btn-ghost inline-flex mr-2" name="customizeBoxPopup">
    <span slot="button" class="icon-[fluent--wand-16-regular] w-5 h-5" />
    <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
      <CustomizeBox />
    </div>
  </RxPopup>
  {#if !$initConfig.processing}
    {#if $user.isLoggedIn}
      <RxPopup class="btn-square btn-ghost inline-flex" name="userBoxPopup" popupWidth={64}>
        <div class="avatar" slot="button">
          <div
            class="w-8 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
          >
            {#if userFullInfo?.cover_path}
              <RxImage src={userFullInfo.cover_path} loading={loadingAvatar} />
            {:else}
              <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
            {/if}
          </div>
        </div>
        <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
          <UserBox />
        </div>
        {#if $game.showGameNav}
          <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
            <TeamBox {canTakePartIn} />
          </div>
        {/if}
      </RxPopup>
    {:else}
      <RxLink href="/account/login" exactlyMatched>
        <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
        <span>{$i18n.t('account.login')}</span>
      </RxLink>
    {/if}
    {#if $game.showGameNav && !$game.team && canTakePartIn && !$user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)}
      <RxLink href={`/games/${$game.current?.id}/participate`} justify="start">
        <span class="w-6 h-6 icon-[fluent--thumb-like-16-regular]" />
        {$i18n.t('games.takePartIn')}
      </RxLink>
    {/if}
  {/if}
</div>
