<script lang="ts">
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
  import { onDestroy } from 'svelte'

  let canSubmitWriteup = false
  const unsubscribe = game.subscribe((val) => {
    if (val.current && val.team) {
      if (val.current.archive_time * 1000 >= Date.now() && val.current.end_time * 1000 <= Date.now()) {
        canSubmitWriteup = true
      }
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>

<div class="min-h-full flex flex-col p-6">
  {#if canSubmitWriteup}
    <RxLink href="/games/{$game.current?.id}/writeups/publish">{$i18n.t('game.submitWriteup')}</RxLink>
  {/if}
  {#if $game.current && $game.current.end_time * 1000 >= Date.now()}
    <div class="flex-1 flex flex-col space-y-4 items-center justify-center opacity-60">
      <span class="icon-[fluent--dismiss-circle-16-regular] w-8 h-8"></span>
      <span>{$i18n.t('game.gameNotEndedYet')}</span>
    </div>
  {:else if $game.current && $game.current.archive_time * 1000 >= Date.now()}
    <div class="flex-1 flex flex-col space-y-4 items-center justify-center opacity-60">
      <span class="icon-[fluent--dismiss-circle-16-regular] w-8 h-8"></span>
      <span>{$i18n.t('game.gameNotArchiveYet')}</span>
    </div>
  {/if}
</div>
