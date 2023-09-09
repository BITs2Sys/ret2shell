<script lang="ts">
  import '$lib/styles/base.sass'
  import '@fontsource/jetbrains-mono'
  import 'overlayscrollbars/overlayscrollbars.css'
  import TitleBar from '$lib/blocks/TitleBar.svelte'
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

  let platformTyped = ''
  let animation = $page.route.id === '/'

  getPlatformInfo().then((data) => {
    if (!data.ok && data.status === 404) {
      goto('/init')
      return
    } else if (!data.ok) {
      // TODO: warn user
    }
    data.json().then((data) => {
      platform.update((value) => {
        return { ...value, ...data }
      })
    })
  })

  onMount(() => {
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
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-light' : 'os-theme-dark', autoHide: 'scroll' },
  }}
  class="relative w-screen h-screen print:h-auto print:overflow-auto"
  defer
>
  <div class="w-full min-h-full flex flex-col">
    <TitleBar />
    <slot />
  </div>
</OverlayScrollbarsComponent>

{#if animation}
  <div class="fixed top-0 left-0 w-screen h-screen bg-base-100 z-50" transition:fade={{ duration: 300 }}>
    <div class="w-full h-full flex flex-col items-center pt-16 pb-24">
      <div class="flex-1" />
      <h1 class="text-3xl font-semibold">{platformTyped}<span class="text-primary animate-ping">_</span></h1>
      <div class="text-xl opacity-0 mt-8">_</div>
      <div class="flex-1" />
    </div>
  </div>
{/if}
