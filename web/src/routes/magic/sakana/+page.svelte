<script lang="ts">
  import 'sakana-widget/lib/index.css'
  import SakanaWidget, { type SakanaWidgetCharacter } from 'sakana-widget'
  import xdsecMascot from '$lib/assets/xdsec-mascot.webp'
  import vidarMascot from '$lib/assets/vidar-mascot.webp'
  import cnssMascot from '$lib/assets/cnss-mascot.webp'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import { onMount } from 'svelte'

  onMount(() => {
    const xdsec = SakanaWidget.getCharacter('takina')
    const vidar = SakanaWidget.getCharacter('chisato')
    const cnss = SakanaWidget.getCharacter('takina')
    if (xdsec) xdsec.image = xdsecMascot
    if (vidar) vidar.image = vidarMascot
    if (cnss) cnss.image = cnssMascot
    SakanaWidget.registerCharacter('xdsec', xdsec as SakanaWidgetCharacter)
    SakanaWidget.registerCharacter('vidar', vidar as SakanaWidgetCharacter)
    SakanaWidget.registerCharacter('cnss', cnss as SakanaWidgetCharacter)
    new SakanaWidget({
      character: 'xdsec',
      controls: false,
      size: 350,
      stroke: {
        color: '#80808060',
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount('#xdsec-box')
    new SakanaWidget({
      character: 'vidar',
      controls: false,
      size: 350,
      stroke: {
        color: '#80808060',
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount('#vidar-box')
    new SakanaWidget({
      character: 'cnss',
      controls: false,
      size: 350,
      stroke: {
        color: '#80808060',
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount('#cnss-box')
  })
</script>

<svelte:head><title>{$i18n.t('surprise.sakana.title')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row items-center justify-center">
  <div class="relative">
    <div id="xdsec-box" />
    <RxLink class="absolute left-1/3 right-1/3 -bottom-3 transform normal-case z-[10]" href="https://l.xdsec.org">
      L-team
    </RxLink>
  </div>
  <div class="relative hidden sm:block">
    <div id="vidar-box" />
    <RxLink class="absolute left-1/3 right-1/3 -bottom-3 transform normal-case z-[10]" href="https://vidar.club">
      Vidar
    </RxLink>
  </div>
  <div class="relative hidden lg:block">
    <div id="cnss-box" />
    <RxLink class="absolute left-1/3 right-1/3 -bottom-3 transform normal-case z-[10]" href="https://cnss.io">
      CNSS
    </RxLink>
  </div>
</div>
<div class="h-24 self-center text-zinc-500 text-center">
  <span>{$i18n.t('surprise.sakana.illustration')} (C)</span>
  <a class="link" href="https://twitter.com/LAttic1ng">Ac4ae0</a>
  <span>,</span>
  <a class="link" href="https://twitter.com/arttnba3">arttnba3</a>
  <br />
  <span>{$i18n.t('surprise.source')}:</span>
  <a class="link" href="https://lab.magiconch.com/sakana/">https://lab.magiconch.com/sakana</a>
</div>
