<script lang="ts">
  import { getChallengeSolvedUser } from '$lib/api/challenge'
  import RxTag from '$lib/components/RxTag.svelte'
  import RxToast from '$lib/components/RxToast.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import type { SubmissionOnlyUserInfo } from '$lib/models/submission'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'

  let clazz = ''
  export { clazz as class }
  export let challenge: Challenge | null
  let solved: SubmissionOnlyUserInfo[] = []

  $: classes = `w-full flex-1 relative overflow-hidden ${clazz}`

  $: if (challenge)
    getChallengeSolvedUser(challenge?.id, 1, 20).then((data) => {
      solved = data.users
    })
</script>

<div class={classes}>
  <div class="absolute w-full h-full">
    <OverlayScrollbarsComponent
      options={{
        scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
      }}
      class="w-full h-full relative print:hidden"
      defer
    >
      {#if !solved || solved.length === 0}
        <p class="w-full min-h-full flex-1 flex flex-row justify-center items-center font-bold opacity-60">
          {$i18n.t('playground.emptyContent')}
        </p>
      {:else}
        <div class="flex flex-col p-4 space-y-2">
          <div class="flex flex-row items-center space-x-2 bg-warning/10 p-4 rounded-lg">
            <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-warning flex-shrink-0" />
            <span>
              {$i18n.t('playground.challengeSolvedWarning')}
            </span>
          </div>
          <div class="flex flex-wrap">
            {#each solved as item}
              <RxTag class="m-2" title={new Date(item.created_at * 1000).toLocaleString()}>
                <a class="hover:underline" href={`/users/${item.user_id}`}>{item.user_name}</a>
              </RxTag>
            {/each}
          </div>
        </div>
      {/if}
    </OverlayScrollbarsComponent>
  </div>
</div>
