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
  import InstanceBox from './InstanceBox.svelte'
  import Engine from '$lib/assets/engine.svelte'
  import LogoAnimate from '$lib/assets/logo-animate.svelte'
  import RxTimer from '$lib/components/RxTimer.svelte'
  import RxPing from '$lib/components/RxPing.svelte'

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

  let loadingAvatar = false

  let gameLastTime = 0

  function calcTime() {
    let now = new Date()
    gameLastTime = ($game.current?.end_time || 0) - now.getTime() / 1000
  }

  setInterval(() => {
    calcTime()
  }, 1000)

  onMount(() => {
    loadingAvatar = true
    userInfo().then(() => {
      loadingAvatar = false
    })
  })
</script>

<div id="page-top" />
<div
  class="navbar h-16 border-b border-b-base-content/5 w-auto backdrop-blur bg-neutral/60 z-50 print:hidden px-2 py-0 sticky top-0 left-0"
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
      <span class="hidden sm:inline">
        <LogoAnimate width={28} height={28} />
      </span>
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
  {#if $game.current && gameLastTime >= 0}
    <div class="px-4 hidden sm:inline">
      <RxTimer time={gameLastTime} />
    </div>
  {/if}
  <RxPopup
    class="btn-square btn-ghost inline-flex mr-2 relative"
    name="customizeBoxPopup"
    on:click={() => {
      $platform.see_custom_box = true
    }}
  >
    <div slot="button">
      <span class="icon-[fluent--wand-16-regular] w-5 h-5" />
      {#if !$platform.see_custom_box}
        <RxPing level="info" />
      {/if}
    </div>
    <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
      <CustomizeBox />
    </div>
  </RxPopup>
  {#if $game.runningInstance}
    <RxPopup class="btn-square btn-ghost inline-flex mr-2" name="instanceBoxPopup" popupWidth={72}>
      <div slot="button" class="text-success">
        <Engine width={32} height={32} />
      </div>
      <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full">
        <InstanceBox />
      </div>
    </RxPopup>
  {/if}
  {#if !$initConfig.processing}
    {#if $user.isLoggedIn}
      <RxPopup class="btn-square btn-ghost inline-flex" name="userBoxPopup" popupWidth={64}>
        <div class="avatar" slot="button">
          <div
            class="w-8 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
          >
            {#if $user.info?.cover_path}
              <RxImage src={$user.info.cover_path} loading={loadingAvatar} />
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
        <span class="hidden sm:inline">{$i18n.t('account.login')}</span>
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
