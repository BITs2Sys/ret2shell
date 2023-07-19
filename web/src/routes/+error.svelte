<script lang="ts">
  import { page } from '$app/stores'
  import { i18n } from '$lib/i18n'

  const messages: Record<number, string> = {
    401: $i18n.t('errors.401'),
    403: $i18n.t('errors.403'),
    404: $i18n.t('errors.404'),
    418: $i18n.t('errors.418'),
    500: $i18n.t('errors.500'),
  }

  const tips: Record<number, string> = {
    401: $i18n.t('errors.401Tip'),
    403: $i18n.t('errors.403Tip'),
    404: $i18n.t('errors.404Tip'),
    418: $i18n.t('errors.418Tip'),
    500: $i18n.t('errors.500Tip'),
  }

  $: message = messages[$page.status] || $i18n.t('errors.unknown')
  $: tip = tips[$page.status] || $i18n.t('errors.unknownTip')
</script>

<div class="flex-1 flex flex-col items-center justify-center">
  <h1 class="font-bold text-3xl"><span class="opacity-60">{$page.status}</span> {message}</h1>
  <p class="opacity-60 mt-8">{tip}</p>
</div>
