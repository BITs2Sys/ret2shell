<script lang="ts">
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import { user } from '$lib/stores/user'
</script>

<li>
  <RxLink ghost justify="start" href="/playground">
    <span class="icon-[fluent--dumbbell-16-regular] w-6 h-6" />
    {$i18n.t('playground.title')}
  </RxLink>
</li>
<li>
  <RxLink ghost justify="start" href="/games">
    <span class="icon-[fluent--flag-16-regular] w-6 h-6" />
    {$i18n.t('games.title')}
  </RxLink>
</li>
<li>
  <RxLink ghost justify="start" href="/announcements">
    <span class="icon-[fluent--megaphone-16-regular] w-6 h-6" />
    {$i18n.t('announcements.title')}
  </RxLink>
</li>
<li>
  <RxLink ghost justify="start" href="/certs">
    <span class="icon-[fluent--trophy-16-regular] w-6 h-6" />
    {$i18n.t('certs.title')}
  </RxLink>
</li>
{#if $user.level >= 2}
  <li>
    <RxLink ghost justify="start" href="/admin">
      <span class="icon-[fluent--organization-16-regular] w-6 h-6" />
      {$i18n.t('admin.title')}
    </RxLink>
  </li>
{/if}
