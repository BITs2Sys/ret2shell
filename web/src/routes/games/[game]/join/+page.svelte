<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
</script>

<svelte:head>
  <title>{$i18n.t('game.joinTeam')} - {$game.current?.name}</title>
</svelte:head>
<div></div>
