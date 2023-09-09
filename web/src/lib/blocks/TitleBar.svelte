<script lang="ts">
  import logo from '$lib/assets/logo.svg'
  import { platform } from '$lib/stores/platform'
  import { user } from '$lib/stores/user'
  import GlobalMenu from '$lib/blocks/GlobalMenu.svelte'
  import CustomizeBox from './CustomizeBox.svelte'
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxPopup from '$lib/components/RxPopup.svelte'
  import UserBox from './UserBox.svelte'
  import { initConfig } from '$lib/stores/init'
  import RxButton from '$lib/components/RxButton.svelte'
</script>

<div class="navbar w-auto backdrop-blur shadow bg-neutral/80 transition-shadow z-40 print:hidden px-2 py-0 sticky top-0">
  {#if $initConfig.processing}
    <RxPopup class="btn-square btn-ghost xl:hidden" name="navPopup" popupWidth={64}>
      <span slot="button" class="icon-[fluent--navigation-16-regular] w-5 h-5" />
      <ul class="menu menu-vertical">
        <GlobalMenu />
      </ul>
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
    <RxLink ghost href="/" exactlyMatched>
      <img class="hidden xl:block" width="28" height="28" src={logo} alt="logo" />
      <span>{$platform.name}</span>
    </RxLink>
    <ul class="menu menu-horizontal px-6 space-x-2 hidden xl:flex">
      <GlobalMenu />
    </ul>
  {/if}
  <div class="flex-1" />
  <RxPopup class="btn-square btn-ghost hidden sm:inline-flex mr-2" name="customizeBoxPopup">
    <span slot="button" class="icon-[fluent--wand-16-regular] w-5 h-5" />
    <CustomizeBox />
  </RxPopup>
  {#if !$initConfig.processing}
    {#if $user.isLoggedIn}
      <RxPopup class="btn-square btn-ghost hidden sm:inline-flex mr-2" name="userBoxPopup" popupWidth={64}>
        <!-- TODO: replace with user's avatar if exists -->
        <div class="avatar" slot="button">
          <div
            class="w-8 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
          >
            <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
          </div>
        </div>
        <UserBox />
      </RxPopup>
    {:else}
      <RxLink href="/account/login" exactlyMatched>
        <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
        <span>{$i18n.t('account.login')}</span>
      </RxLink>
    {/if}
  {/if}
</div>
