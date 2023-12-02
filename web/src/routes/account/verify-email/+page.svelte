<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import { page } from '$app/stores'
  import { onMount } from 'svelte'
  import { showMessage } from '$lib/stores/toast'
  import { goto } from '$app/navigation'
  import { verifyEmail } from '$lib/api/account'
  import { platform } from '$lib/stores/platform'
  import RxCard from '$lib/components/RxCard.svelte'
  import LogoAnimate from '$lib/assets/animates/logo-animate.svelte'
  import { fly, blur } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'

  onMount(() => {
    const email = $page.url.searchParams.get('email')
    const token = $page.url.searchParams.get('token')
    if (!email || !token) {
      showMessage('error', $i18n.t('email.invalidVerifyLink'), undefined)
      // goto('/', { replaceState: true })
      return
    }
    verifyEmail({ email, token })
      .then(() => {
        showMessage('success', $i18n.t('email.verifySuccess'), 5000)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('email.verifyFailed')}: ${err.response?.data}`, undefined)
      })
      .finally(() => {
        goto('/', { replaceState: true })
      })
  })
  let show = false
  onMount(() => {
    setTimeout(() => {
      show = true
    }, 100)
  })
</script>

<svelte:head><title>{$i18n.t('account.verifyEmail')} - {$platform.name}</title></svelte:head>

{#if show}
  <div class="flex-1 flex flex-col justify-center items-center space-y-12">
    <div class="flex flex-row items-center space-x-8">
      <div in:fly={{ duration: 1000, x: 32, opacity: 0, easing: quintOut }}>
        <RxCard class="flex flex-col items-center justify-center w-40 h-40">
          <LogoAnimate width={128} height={128} />
        </RxCard>
      </div>
      <span class="loading loading-infinity w-16 text-info" in:blur={{ duration: 300, amount: 20 }}></span>
      <div in:fly={{ duration: 1000, x: -32, opacity: 0, easing: quintOut }}>
        <RxCard class="flex flex-col items-center justify-center w-40 h-40">
          <span class="icon-[fluent--mail-24-filled] w-28 h-28"></span>
        </RxCard>
      </div>
    </div>
    <h2 class="text-2xl font-bold" in:fly={{ duration: 1000, y: -16, opacity: 0, easing: quintOut, delay: 500 }}>
      {$i18n.t('account.verifyingEmail')}
    </h2>
    <div class="h-24"></div>
  </div>
{/if}
