<script lang="ts">
  import { i18n } from '$lib/i18n'
  import RxLink from '$lib/components/RxLink.svelte'
  import { user } from '$lib/stores/user'
  import { Permission } from '$lib/models/user'
  import { game } from '$lib/stores/game'

  $: hasAdminEntry = $user && $user.permissions.some((p) => p >= Permission.Publish)
</script>

<li>
  <RxLink ghost justify="start" href={`/games/${$game.current?.id}/challenges`}>
    <span class="icon-[fluent--braces-16-regular] w-6 h-6" />
    {$i18n.t('games.challenges')}
  </RxLink>
</li>
<li>
  <RxLink ghost justify="start" href={`/games/${$game.current?.id}/scoreboard`}>
    <span class="icon-[fluent--trophy-16-regular] w-6 h-6" />
    {$i18n.t('games.scoreboard')}
  </RxLink>
</li>
{#if new Date(($game.current?.archive_time || 0) * 1000) < new Date()}
<li>
  <RxLink ghost justify="start" href={`/games/${$game.current?.id}/writeups`}>
    <span class="icon-[fluent--book-open-16-regular] w-6 h-6" />
    {$i18n.t('games.writeups')}
  </RxLink>
</li>
{/if}
{#if hasAdminEntry}
  <li>
    <RxLink ghost justify="start" href={`/admin/games/${$game.current?.id}`}>
      <span class="icon-[fluent--organization-16-regular] w-6 h-6" />
      {$i18n.t('admin.title')}
    </RxLink>
  </li>
{/if}
<li>
  <RxLink ghost justify="start" href="/games" exactlyMatched>
    <span class="icon-[fluent--arrow-exit-20-regular] w-6 h-6 text-warning" />
    {$i18n.t('games.exit')}
  </RxLink>
</li>
