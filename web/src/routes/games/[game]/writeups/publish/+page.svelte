<script lang="ts">
  import { goto } from '$app/navigation'
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import {
    createGameTeamWriteUpSelf,
    getGameTeamSolves,
    getGameTeamWriteUpSelf,
    getTeamMembers,
    updateGameTeamWriteUpSelf,
  } from '$lib/api/game'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCodearea from '$lib/components/RxCodearea.svelte'
  import { i18n } from '$lib/i18n'
  import type { User } from '$lib/models/user'
  import type { WriteUp } from '$lib/models/write_up'
  import { game } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onDestroy, onMount } from 'svelte'

  let content = ''
  let loading = false
  let constructingTemplate = false
  let isCreate = false

  onMount(() => {
    if ($game.current) {
      if ($game.current.archive_time * 1000 < Date.now()) {
        showMessage('warning', $i18n.t('game.gameIsArchived'), 5000)
        goto(`/games/${$game.current.id}`, { replaceState: true })
      } else if ($game.current.end_time * 1000 > Date.now()) {
        showMessage('warning', $i18n.t('game.gameNotEnded'), 5000)
        goto(`/games/${$game.current.id}`, { replaceState: true })
      }
    }
  })

  function initNewContent() {
    constructingTemplate = true
    content = `# ${$game.current?.name} Writeup by ${$game.team?.name}\n\n> ${$i18n.t('game.writeupPlaceholder')}\n\n`
    getTagList()
      .then((tags) => {
        getGameTeamSolves($game.current?.id || -1, $game.team?.id || -1)
          .then(async (solves) => {
            let challenges = $game.challenges
            let members: User[] = []
            try {
              if (challenges.length === 0) {
                challenges = (await getChallengeList($game.current?.id || -1, 1, 200)).challenges
              }
              members = await getTeamMembers($game.current?.id || -1, $game.team?.id || -1)
            } catch (err) {
              showMessage('error', `${$i18n.t('challenge.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
              return
            }
            for (let tag of tags) {
              if (
                challenges.filter((x) => x.tag_id === tag.id && solves.find((s) => s.challenge_id === x.id)).length ===
                0
              )
                continue
              content += `\n## ${tag.name}\n\n`
              for (let challenge of challenges.filter((x) => x.tag_id === tag.id)) {
                let solve = solves.find((x) => x.challenge_id === challenge.id)
                if (solve) {
                  content += `### ${challenge.name}\n\n`
                  let user = members.find((x) => x.id === solve?.user_id)
                  content += `> by [${user?.name}](/users/${user?.id})\n\n`
                }
              }
            }
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
          .finally(() => {
            constructingTemplate = false
          })
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        constructingTemplate = false
      })
  }

  const unsubscribe = game.subscribe((val) => {
    if (val.team) {
      loading = true
      getGameTeamWriteUpSelf(val.team.game_id)
        .then((data) => {
          content = data.content
          isCreate = false
        })
        .catch((err) => {
          if ((err as AxiosError).response?.status === 404) {
            initNewContent()
            isCreate = true
          } else showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          loading = false
        })
    }
  })

  function submitWriteup() {
    let writeup: WriteUp = {
      id: 0,
      title: `${$game.current?.name} Writeup by ${$game.team?.name}`,
      published_at: 0,
      updated_at: 0,
      game_id: $game.current?.id || -1,
      team_id: $game.team?.id || -1,
      content: content,
      hidden: false,
      author_id: $user.id,
    }
    if (isCreate) {
      createGameTeamWriteUpSelf($game.current?.id || -1, writeup)
        .then(() => {
          showMessage('success', $i18n.t('game.writeupSubmitSuccess'), 5000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('game.writeupSubmitFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    } else {
      updateGameTeamWriteUpSelf($game.current?.id || -1, writeup)
        .then(() => {
          showMessage('success', $i18n.t('game.writeupSubmitSuccess'), 5000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('game.writeupSubmitFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  onDestroy(() => {
    unsubscribe()
  })
</script>
<svelte:head>
  <title>{$i18n.t('games.submitWriteup')} - {$game.current?.name}</title>
</svelte:head>
<div class="h-16 border-b-2 border-base-content/5 flex flex-row justify-between items-center px-2 backdrop-blur">
  <h1 class="font-bold px-2">{$i18n.t('games.submitWriteup')}</h1>
  <RxButton on:click={submitWriteup}>{$i18n.t('answer.submit')}</RxButton>
</div>
<div class="flex-1 flex flex-row">
  <div class="flex-1 backdrop-blur flex-shrink-0 relative">
    <RxCodearea bind:value={content} lang="markdown"></RxCodearea>
    {#if constructingTemplate || loading}
      <div class="absolute top-0 left-0 w-full h-full flex justify-center items-center backdrop-blur">
        <span class="loading loading-spinner loading-md" />
      </div>
    {/if}
  </div>
  <div class="flex-1 flex-shrink-0 border-l-2 border-base-content/5 relative">
    <div class="absolute w-full h-full top-0 left-0">
      <OverlayScrollbarsComponent
        options={{
          scrollbars: {
            theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light',
            autoHide: 'scroll',
          },
        }}
        class="w-full h-full relative print:hidden"
        defer
      >
        <div class="p-8">
          <RxArticle {content}></RxArticle>
        </div>
      </OverlayScrollbarsComponent>
    </div>
  </div>
</div>
