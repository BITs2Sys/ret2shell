<script lang="ts">
  import { logout } from '$lib/api/account'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { User } from '$lib/models/user'
  import { user, userInfo, userReset } from '$lib/stores/user'
  import { onMount } from 'svelte'

  function handleLogout() {
    logout().finally(() => {
      userReset()
      setTimeout(() => {
        window.location.href = '/'
      }, 100)
    })
  }

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

<div class="p-2 flex flex-col">
  <RxLink justify="start" class="h-16 flex flex-nowrap overflow-hidden" href="/account/profile">
    <div class="avatar">
      <div
        class="w-10 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
      >
        {#if userFullInfo?.cover_path}
          <RxImage src={userFullInfo.cover_path} loading={loadingAvatar} />
        {:else}
          <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
        {/if}
      </div>
    </div>
    <div class="ml-2 flex flex-col items-start overflow-hidden">
      <span class="font-bold text-ellipsis overflow-hidden whitespace-nowrap w-full">{$user.name}</span>
      <span class="opacity-60">0x{$user.id.toString(16).padStart(6, '0')}</span>
    </div>
  </RxLink>
  <div class="divider m-0 ml-2 mr-2" />
  <RxLink href="/account/settings" justify="start" ghost>
    <span class="w-6 h-6 icon-[fluent--settings-16-regular]" />
    {$i18n.t('account.settings')}
  </RxLink>
  <div class="divider m-0 ml-2 mr-2" />
  <RxButton ghost justify="start" on:click={handleLogout}>
    <span class="w-6 h-6 icon-[fluent--arrow-exit-20-regular] text-error" />
    {$i18n.t('account.logout')}
  </RxButton>
</div>
