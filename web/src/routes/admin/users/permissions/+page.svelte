<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
</script>

<svelte:head><title>{$i18n.t('admin.routes.permissions')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col space-y-4 p-6 lg:p-12">
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--emoji-24-regular] w-6 h-6 text-primary"></span>
    <span>{$i18n.t('permission.basic')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.basicIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--checkmark-circle-24-regular] w-6 h-6 text-success"></span>
    <span>{$i18n.t('permission.verified')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.verifiedIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--data-pie-24-regular] w-6 h-6 text-primary"></span>
    <span>{$i18n.t('permission.statistics')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.statisticsIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--calendar-24-regular] w-6 h-6 text-primary"></span>
    <span>{$i18n.t('permission.calendar')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.calendarIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--megaphone-24-regular] w-6 h-6 text-primary"></span>
    <span>{$i18n.t('permission.publish')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.publishIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--eye-24-regular] w-6 h-6 text-warning"></span>
    <span>{$i18n.t('permission.audit')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.auditIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--organization-24-regular] w-6 h-6 text-error"></span>
    <span>{$i18n.t('permission.organize')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.organizeIntro')}</p>
  <h2 class="text-lg font-bold flex flex-row space-x-2 items-center border-b border-b-base-content/10 py-2">
    <span class="icon-[fluent--flash-24-regular] w-6 h-6 text-error"></span>
    <span>{$i18n.t('permission.devops')}</span>
  </h2>
  <p class="text-base opacity-80">{$i18n.t('permission.devopsIntro')}</p>
</div>
