import { getGameTeamSolves, getSelfTeamInfo, getSelfTeamRank } from '$lib/api/game'
import { getSelfRunningInstance } from '$lib/api/instance'
import type { Challenge } from '$lib/models/challenge'
import type { Game } from '$lib/models/game'
import type { Instance } from '$lib/models/instance'
import type { Submission } from '$lib/models/submission'
import type { Team } from '$lib/models/team'
import type { AxiosError } from 'axios'
import { get, writable } from 'svelte/store'
import { showMessage } from './toast'
import { i18n } from '$lib/i18n'

class GameStore {
  current: Game | null
  cached: Game | null
  team: Team | null
  rank: number | null
  challenges: Challenge[]
  submissions: Submission[]
  runningInstance: Instance | null
  showGameNav: boolean

  constructor() {
    this.current = null
    this.cached = null
    this.team = null
    this.rank = null
    this.challenges = []
    this.submissions = []
    this.showGameNav = false
    this.runningInstance = null
  }

  inProgress(): boolean {
    const now = new Date().getTime() / 1000
    return !!this.current && this.current.start_time < now && this.current.end_time > now
  }
}

export const game = writable(new GameStore())

export function refreshInstanceState() {
  getSelfRunningInstance()
    .then((resp) => {
      game.update((g) => {
        g.runningInstance = resp
        return g
      })
    })
    .catch(() => {
      game.update((g) => {
        g.runningInstance = null
        return g
      })
    })
}

export function refreshTeam() {
  const gameObj = get(game).current
  if (gameObj) {
    getSelfTeamInfo(gameObj.id)
      .then((res) => {
        game.update((value) => {
          value.team = res
          return value
        })
        getGameTeamSolves(gameObj.id, res.id)
          .then((res) => {
            game.update((value) => {
              value.submissions = res
              return value
            })
          })
          .catch((err) => {
            showMessage(
              'error',
              `${get(i18n).t('playground.fetchSelfSubmissionsFailed')}: ${(err as AxiosError).response?.data}`,
              5000
            )
          })

        getSelfTeamRank(gameObj.id)
          .then((r) => {
            game.update((value) => {
              value.rank = r.rank
              return value
            })
          })
          .catch((err) => {
            showMessage(
              'error',
              `${get(i18n).t('game.fetchTeamRankFailed')}: ${(err as AxiosError).response?.data}`,
              5000
            )
          })
      })
      .catch((err) => {
        if ((err as AxiosError).response?.status !== 404) {
          showMessage('error', `${get(i18n).t('games.fetchTeamError')}: ${(err as AxiosError).response?.data}`, 5000)
        }
        game.update((value) => {
          value.team = null
          return value
        })
      })
  }
}
