<script lang="ts">
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
</script>

<div class="flex flex-col space-y-2 p-4">
  <RxLink href="/account/settings/info" ghost justify="start">
    <span class="icon-[fluent--info-20-regular] w-5 h-5"></span>
    <span>{$i18n.t('account.infoSetting')}</span>
  </RxLink>
  <RxLink href="/account/settings/password" ghost justify="start">
    <span class="icon-[fluent--password-20-regular] w-5 h-5"></span>
    <span>{$i18n.t('account.passwordSetting')}</span>
  </RxLink>
  <RxLink href="/account/settings/institute" ghost justify="start">
    <span class="icon-[fluent--people-team-20-regular] w-5 h-5"></span>
    <span>{$i18n.t('account.instituteSetting')}</span>
  </RxLink>
  <RxLink href="/account/settings/mov-rbp-rsp-pop-rbp" ghost justify="start" class="text-error">
    <span class="icon-[fluent--person-walking-20-regular] w-5 h-5 -rotate-90"></span>
    <span>{$i18n.t('account.dangerousSetting')}</span>
  </RxLink>
</div>
