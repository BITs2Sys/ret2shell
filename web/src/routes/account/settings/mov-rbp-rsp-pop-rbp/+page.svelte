<script lang="ts">
  import { goto } from '$app/navigation'
  import { deactivateAccount } from '$lib/api/account'
  import Background from '$lib/blocks/Background.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import { user, userReset } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { blur, fade } from 'svelte/transition'

  let name: string = ''
  let deactivated = false
  $: canDelete = name === $user.name

  function handleDeactivate() {
    if (canDelete) {
      deactivateAccount()
        .then(() => {
          userReset()
          deactivated = true
          setTimeout(() => {
            goto('/')
          }, 3000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('account.deleteAccountFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }
</script>

<svelte:head><title>{$i18n.t('account.bye')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="pt-4 md:p-12 md:pb-4 flex flex-row md:flex-col items-center justify-center">
      <span class="icon-[fluent--warning-24-filled] text-error w-6 h-6 md:w-24 md:h-24"></span>
      <h1 class="text-center text-lg font-bold text-error ml-4 md:ml-0 md:mt-4">
        {$i18n.t('account.deleteAccountTitle')}
      </h1>
    </div>
    <div class="divider"></div>
    <article class="prose w-full max-w-5xl self-center mt-4">
      <p><strong>{$i18n.t('account.deleteAccountTips1')}</strong></p>
      <ul>
        <li>{$i18n.t('account.deleteAccountTipsList1')}</li>
        <li>{$i18n.t('account.deleteAccountTipsList2')}</li>
        <li>{$i18n.t('account.deleteAccountTipsList3')}</li>
        <li>{$i18n.t('account.deleteAccountTipsList4')}</li>
      </ul>
      <p><strong>{$i18n.t('account.deleteAccountTips3')}</strong></p>
      <p class="text-error">{$i18n.t('account.deleteAccountTips2')}</p>
    </article>
    <div class="divider"></div>
    <div class="w-full max-w-5xl self-center flex flex-row items-center justify-center">
      <RxInput icon="icon-[fluent--person-20-regular] w-5 h-5" class="w-0" bind:value={name}>
        <RxButton class="join-item ml-0 text-error" disabled={!canDelete} on:click={handleDeactivate}>
          <span class="icon-[fluent--arrow-exit-20-regular] w-5 h-5"></span>
          <span class="icon-[fluent--person-walking-20-regular] w-5 h-5"></span>
          <span class="hidden md:inline">{$i18n.t('account.bye')}</span>
        </RxButton>
      </RxInput>
    </div>
    <div class="divider"></div>
  </div>
</div>
{#if deactivated}
  <div
    class="fixed top-0 left-0 w-full h-full bg-base-100 flex flex-col justify-center items-center z-50"
    transition:blur={{ amount: 20, duration: 300 }}
  >
    <Background />
    <h1 class="text-2xl font-bold" transition:fade={{ duration: 1000, delay: 1000 }}>{$i18n.t('account.bye')}</h1>
  </div>
{/if}
