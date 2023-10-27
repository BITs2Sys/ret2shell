<script lang="ts">
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import { createEventDispatcher } from 'svelte'
  const dispatch = createEventDispatcher()

  let clazz = ''
  export { clazz as class }

  let currentChallenge: Challenge | null = null
</script>

<ExtraPanel title={currentChallenge?.name} on:close={() => dispatch('close')} class={clazz}></ExtraPanel>
