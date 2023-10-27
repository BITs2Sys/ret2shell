<script lang="ts">
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import { i18n } from '$lib/i18n'
  import { createEventDispatcher } from 'svelte'
  const dispatch = createEventDispatcher()

  let clazz = ''
  export { clazz as class }
</script>

<ExtraPanel title={$i18n.t('admin.createChallenge')} on:close={() => dispatch('close')} class={clazz}></ExtraPanel>
