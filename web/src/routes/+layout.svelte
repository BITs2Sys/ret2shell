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
  import { getPlatformInfo } from '$lib/api/platform'
  import { goto } from '$app/navigation'
  import { showMessage } from '$lib/stores/toast'
  import { i18n } from '$lib/i18n'
  import GlobalToast from '$lib/blocks/GlobalToast.svelte'
  import type { AxiosError } from 'axios'
  import { refreshUserInfo, user, userReset } from '$lib/stores/user'

  let platformTyped = ''
  let animation = $page.url.pathname === '/'

  getPlatformInfo()
    .then((data) => {
      platform.update((value) => {
        return {
          accept_cookies: value.accept_cookies,
          ...data,
        }
      })
    })
    .catch((err) => {
      if ((err as AxiosError).response?.status === 404) {
        userReset()
        return goto('/init')
      } else showMessage('error', $i18n.t('platform.backendOffline'))
    })

  refreshUserInfo()

  function acceptCookiePolicy() {
    $platform.accept_cookies = true
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
</script>

<svelte:head><title>{$platform.name}</title></svelte:head>
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
