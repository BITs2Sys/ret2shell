<script lang="ts">
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { user } from '$lib/stores/user'
</script>

<div class="p-2 flex flex-col">
  <RxLink ghost justify="start" class="h-16" href="/account/profile">
    <div class="avatar">
      <div
        class="w-10 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
      >
        <span class="w-6 h-6 icon-[fluent--person-16-regular]" />
      </div>
    </div>
    <div class="ml-2 flex flex-col items-start">
      <span class="font-bold">{$user.name}</span>
      <span class="opacity-60">0x{$user.id.toString(16).padStart(6, '0')}</span>
    </div>
  </RxLink>
  <div class="divider m-0 ml-2 mr-2" />
  <RxLink href="/account/settings" justify="start" ghost>
    <span class="w-6 h-6 icon-[fluent--settings-16-regular]" />
    {$i18n.t('account.settings')}
  </RxLink>
</div>
