<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import LogoAnimate from '$lib/assets/animates/logo-animate.svelte'
  import GitHub from '$lib/assets/brands/github.svelte'
  import Google from '$lib/assets/brands/google.svelte'
  import GitLab from '$lib/assets/brands/gitlab.svelte'
  import QQ from '$lib/assets/brands/qq.svelte'
  import XDU from '$lib/assets/brands/xdu.svelte'
  import { queryParam } from 'sveltekit-search-params'
  import RxCard from '$lib/components/RxCard.svelte'
  import { fly, blur } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'
  import { onMount } from 'svelte'

  let icon = queryParam('provider')
  $: iconSource = {
    github: GitHub,
    google: Google,
    gitlab: GitLab,
    qq: QQ,
    xdu: XDU,
  }[$icon || 'github']

  let show = false
  onMount(() => {
    setTimeout(() => {
      show = true
    }, 100)
  })
</script>

<svelte:head><title>{$i18n.t('account.3rdAuth')} - {$platform.name}</title></svelte:head>

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
          <svelte:component this={iconSource} width={128} height={128} />
        </RxCard>
      </div>
    </div>
    <h2 class="text-2xl font-bold" in:fly={{ duration: 1000, y: -16, opacity: 0, easing: quintOut, delay: 500 }}>
      {$i18n.t('account.connectOAuthServer')}
    </h2>
    <div class="h-24"></div>
  </div>
{/if}
