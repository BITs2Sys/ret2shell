import { getSelfRunningInstance } from '$lib/api/instance'
import type { Challenge } from '$lib/models/challenge'
import type { Game } from '$lib/models/game'
import type { Instance } from '$lib/models/instance'
import type { Submission } from '$lib/models/submission'
import type { Team } from '$lib/models/team'
import { writable } from 'svelte/store'

class GameStore {
  current: Game | null
  cached: Game | null
  team: Team | null
  challenges: Challenge[]
  submissions: Submission[]
  runningInstance: Instance | null
  showGameNav: boolean

  constructor() {
    this.current = null
    this.cached = null
    this.team = null
    this.challenges = []
    this.submissions = []
    this.showGameNav = false
    this.runningInstance = null
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
