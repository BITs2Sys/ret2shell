<script lang="ts">
  import { onDestroy, type ComponentType } from 'svelte'
  import Welcome from './Welcome.svelte'
  import WhatIsCtf from './WhatIsCTF.svelte'
  import F12 from './F12.svelte'
  import TheEnd from './TheEnd.svelte'
  import F12Ok from './F12Ok.svelte'
  import FutherLearning from './FutherLearning.svelte'
  import PlatformIntro from './PlatformIntro.svelte'
  import { step } from './store'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'

  const steps: ComponentType[] = [Welcome, WhatIsCtf, F12, F12Ok, FutherLearning, PlatformIntro, TheEnd]

  $: current = steps[$step]

  onDestroy(() => {
    $step = 0
  })
</script>

<svelte:head>
  <title>{$i18n.t('surprise.tutorial.title')} - {$platform.name}</title>
</svelte:head>
<div class="flex-1 flex flex-col">
  <svelte:component this={current} />
</div>
