<script lang="ts">
  import Grass from '$lib/assets/animates/grass.svelte'
  import RxCard from '$lib/components/RxCard.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import Thankyoujs from '$lib/assets/imgs/thankyoujs.png'
  import xdsecMascot from '$lib/assets/imgs/xdsec-mascot.webp'
  import LogoAnimate from '$lib/assets/animates/logo-animate.svelte'
  import RxToast from '$lib/components/RxToast.svelte'

  let showToast = false
  let toast = ''
  let availableToasts = [
    $i18n.t('surprise.toast.0'),
    $i18n.t('surprise.toast.1'),
    $i18n.t('surprise.toast.2'),
    $i18n.t('surprise.toast.3'),
    $i18n.t('surprise.toast.4'),
    $i18n.t('surprise.toast.5'),
    $i18n.t('surprise.toast.6'),
  ]
  function showRandomToast() {
    if (showToast) return
    toast = availableToasts[Math.floor(Math.random() * availableToasts.length)]
    showToast = true
    setTimeout(() => {
      showToast = false
    }, 3000)
  }
</script>

<svelte:head>
  <title>{$i18n.t('surprise.title')} - {$platform.name}</title>
</svelte:head>

<div class="flex-1 flex flex-col p-4 lg:p-6 items-center">
  <div class="flex flex-col max-w-5xlf relative">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <Grass width={128} height={128} />
      <button class="text-4xl font-bold animate-bounce" on:click={showRandomToast}>{$i18n.t('surprise.title')}</button>
      <Grass width={128} height={128} />
    </div>
    {#if showToast}
      <div class="absolute -bottom-8 left-1/2 -translate-x-1/2">
        <RxToast id="0" message={toast}></RxToast>
      </div>
    {/if}
  </div>
  <div class="flex-1 flex flex-row space-x-12 items-center">
    <RxCard>
      <div class="flex flex-col items-center space-y-4">
        <LogoAnimate width={320} height={320} />
        <h2 class="text-xl font-bold">{$i18n.t('surprise.tutorial.title')}</h2>
        <RxLink class="w-full" href="/magic/tutorial">ENTER</RxLink>
      </div>
    </RxCard>
    <RxCard>
      <div class="flex flex-col items-center space-y-4">
        <img class="rounded-lg h-80" src={xdsecMascot} alt="" />
        <h2 class="text-xl font-bold">{$i18n.t('surprise.sakana.title')}</h2>
        <RxLink class="w-full" href="/magic/sakana">ENTER</RxLink>
      </div>
    </RxCard>
    <RxCard>
      <div class="flex flex-col items-center space-y-4">
        <img class="rounded-lg h-80" src={Thankyoujs} alt="" />
        <h2 class="text-xl font-bold">{$i18n.t('surprise.jswdnmd.title')}</h2>
        <RxLink class="w-full" href="/magic/wdnmd">ENTER</RxLink>
      </div>
    </RxCard>
  </div>
</div>
