<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxButton from '$lib/components/RxButton.svelte'
  import type { Game } from '$lib/models/game'

  let games: Game[] = []
</script>

<svelte:head><title>{$i18n.t('games.title')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="w-72 xl:w-80 hidden lg:flex flex-col xl:ml-12 mt-6">
    <RxButton />
  </div>
  <div class="flex-1 max-w-5xl flex flex-col p-3 lg:p-6" />
  <div class="w-80 hidden 2xl:flex mr-12" />
</div>
