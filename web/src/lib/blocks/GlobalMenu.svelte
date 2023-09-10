<script lang="ts">
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import { user } from '$lib/stores/user'
    import { Permission } from '$lib/models/user'

  $: hasAdminEntry = $user.permissions.some((p) => p >= Permission.Publish)
</script>

<li>
  <RxLink ghost justify="start" href="/wiki">
    <span class="icon-[fluent--book-number-16-regular] w-6 h-6" />
    {$i18n.t('wiki.title')}
  </RxLink>
</li>
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
{#if hasAdminEntry}
  <li>
    <RxLink ghost justify="start" href="/admin">
      <span class="icon-[fluent--organization-16-regular] w-6 h-6" />
      {$i18n.t('admin.title')}
    </RxLink>
  </li>
{/if}
