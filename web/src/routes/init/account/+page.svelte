<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
</script>

<svelte:head><title>{$i18n.t('init.accountTitle')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.accountTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
  </div>
</div>
