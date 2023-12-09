<script lang="ts">
  import { i18n } from '$lib/i18n'

  export let status: number = -1

  const messages: Record<number, string> = {
    401: $i18n.t('errors.401'),
    403: $i18n.t('errors.403'),
    404: $i18n.t('errors.404'),
    418: $i18n.t('errors.418'),
    500: $i18n.t('errors.500'),
    502: $i18n.t('errors.502'),
  }

  const tips: Record<number, string> = {
    401: $i18n.t('errors.401Tip'),
    403: $i18n.t('errors.403Tip'),
    404: $i18n.t('errors.404Tip'),
    418: $i18n.t('errors.418Tip'),
    500: $i18n.t('errors.500Tip'),
    502: $i18n.t('errors.502Tip'),
  }

  $: message = messages[status] || $i18n.t('errors.unknown')
  $: tip = tips[status] || $i18n.t('errors.unknownTip')
</script>

<div class="flex-1 flex flex-col items-center justify-center">
  <h1 class="font-bold text-3xl">
    <span class="opacity-60">{status}</span>
    {message}
  </h1>
  <p class="opacity-60 mt-8">{tip}</p>
</div>
