<script lang="ts">
    import RxInput from "$lib/components/RxInput.svelte"
    import { i18n } from "$lib/i18n"

</script>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="w-72 xl:w-80 hidden lg:flex flex-col xl:ml-12 mt-6 sticky top-28 h-min">
    <RxInput placeholder={$i18n.t('playground.searchPlaceholder')} />
  </div>
  <div class="flex-1 max-w-5xl">
    <slot />
  </div>
  <div class="w-80 hidden 2xl:flex mr-12" />
</div>
