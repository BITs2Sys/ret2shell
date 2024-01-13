<script lang="ts">
  import '$lib/styles/base.scss'
  import '@fontsource/jetbrains-mono'
  import 'overlayscrollbars/overlayscrollbars.css'
  import TitleBar from '$lib/blocks/title/TitleBar.svelte'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import Background from '$lib/blocks/Background.svelte'
  import { theme } from '$lib/stores/theme'
  import { onDestroy } from 'svelte'
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import { platform } from '$lib/stores/platform'
  import { page } from '$app/stores'
  import { getPlatformInfo, getPlatformVersion } from '$lib/api/v1/platform'
  import { goto } from '$app/navigation'
  import { removeMessage, showMessage } from '$lib/stores/toast'
  import { i18n } from '$lib/i18n'
  import GlobalToast from '$lib/blocks/GlobalToast.svelte'
  import type { AxiosError } from 'axios'
  import { refreshUserInfo, user, userReset } from '$lib/stores/user'
  import { Permission } from '$lib/models/user'
  import { resendEmailVerification } from '$lib/api/v1/account'

  let platformTyped = ''
  let animation = $page.url.pathname === '/'

  getPlatformInfo()
    .then((data) => {
      platform.update((value) => {
        return {
          accept_cookies: value.accept_cookies,
          see_custom_box: value.see_custom_box,
          see_magic_category: value.see_magic_category,
          version: value.version,
          ...data,
        }
      })
    })
    .catch((err) => {
      if ((err as AxiosError).response?.status === 404) {
        userReset()
        return goto('/init', { replaceState: true })
      } else showMessage('error', $i18n.t('platform.backendOffline'))
    })

  getPlatformVersion()
    .then((res) => {
      $platform.version = res
      console.log(
        `\n%cR%cet %c2 %cS%chell %cv%c${$platform.version}\n\n%cCopyright (c) 2022 - 2024 %cRet 2 Shell%c, All rights reserved.\n`,
        'color: #0078D6; font-weight: bold; font-size: 1.5rem;',
        'color: currentColor; font-weight: bold; font-size: 1.5rem;',
        'color: #808080; font-weight: bold; font-size: 1.5rem;',
        'color: #f83030; font-weight: bold; font-size: 1.5rem;',
        'color: currentColor; font-weight: bold; font-size: 1.5rem;',
        'color: #0078D6',
        'color: #808080',
        'color: #808080',
        'color: #808080;text-decoration: underline;',
        'color: #808080;'
      )
      console.log(
        `\n%cHaving issue? You can open a ticket on https://github.com/ret2shell, any bug reports or feature requests are welcome.\n`,
        'color: currentColor;'
      )
      console.log(
        `\n%cIf you want to self-host CTF platforms or look for further cooperating, please contact ret2shell@woooo.tech.\n`,
        'color: currentColor;'
      )
    })
    .catch((err) => {
      showMessage(
        'error',
        $i18n.t('platform.failedToFetchPlatformVersion') + ': ' + (err as AxiosError).response?.data,
        5000
      )
    })

  refreshUserInfo()

  function acceptCookiePolicy() {
    $platform.accept_cookies = true
  }

  function sendEmailVerification() {
    resendEmailVerification()
      .then(() => {
        showMessage('success', $i18n.t('email.resent'), 5000)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('email.sendFailed')}: ${err.response?.data}`, 5000)
      })
  }

  onMount(() => {
    if (!$platform.accept_cookies) {
      showMessage(
        'info',
        $i18n.t('platform.cookieAlert'),
        undefined,
        acceptCookiePolicy,
        $i18n.t('action.ok'),
        acceptCookiePolicy,
        $i18n.t('action.yes')
      )
    }

    if ($user.token && !$user.permissions.includes(Permission.Verified)) {
      showMessage(
        'warning',
        $i18n.t('account.needVerify'),
        undefined,
        undefined,
        undefined,
        sendEmailVerification,
        $i18n.t('account.resendVerifyEmail')
      )
    }

    if (!animation) return
    setTimeout(() => {
      let platformLast = `\xa0\xa0[ ${$platform.name} ] `
      const timer = setInterval(() => {
        if (platformLast === '') {
          clearInterval(timer)
          setTimeout(() => {
            animation = false
          }, 500)
        }
        platformTyped = platformTyped + platformLast.charAt(0)
        platformLast = platformLast.slice(1)
      }, 50)
    }, 1000)
  })

  const themeUnsubscribe = theme.subscribe((value) => {
    document.documentElement.setAttribute('data-theme', value.colorScheme)
  })

  onDestroy(themeUnsubscribe)

  let screenWidth: number
  let screenHeight: number
  let screenTip: string | null = null
  function watchScreenSize(width: number, height: number) {
    if (width < 1600 || height < 900) {
      if (!screenTip) screenTip = showMessage('warning', $i18n.t('platform.screenTooSmall'))
    } else if (screenTip) {
      removeMessage(screenTip)
      screenTip = null
    }
  }
  $: watchScreenSize(screenWidth, screenHeight)
</script>

<svelte:head><title>{$platform.name}</title></svelte:head>
<svelte:window bind:innerWidth={screenWidth} bind:innerHeight={screenHeight} />
<Background />
<OverlayScrollbarsComponent
  options={{
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
  }}
  class="relative w-screen h-screen print:h-auto print:overflow-auto"
  defer
>
  <div class="w-full min-h-full flex flex-col">
    <TitleBar />
    <slot />
  </div>
</OverlayScrollbarsComponent>
<GlobalToast />

{#if animation}
  <div class="fixed top-0 left-0 w-screen h-screen bg-base-100 z-50" transition:fade={{ duration: 300 }}>
    <Background />
    <div class="w-full h-full flex flex-col items-center pt-16 pb-24">
      <div class="flex-1" />
      <h1 class="text-3xl font-semibold">
        {platformTyped}
        <span class="text-primary animate-ping">_</span>
      </h1>
      <div class="text-xl opacity-0 mt-8">_</div>
      <div class="flex-1" />
    </div>
  </div>
{/if}
