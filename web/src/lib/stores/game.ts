import type { Challenge } from '$lib/models/challenge'
import type { Game } from '$lib/models/game'
import type { Submission } from '$lib/models/submission'
import type { Team } from '$lib/models/team'
import { writable } from 'svelte/store'

class GameStore {
  current: Game | null
  cached: Game | null
  team: Team | null
  challenges: Challenge[]
  submissions: Submission[]
  showGameNav: boolean

  constructor() {
    this.current = null
    this.cached = null
    this.team = null
    this.challenges = []
    this.submissions = []
    this.showGameNav = false
  }
}

export const game = writable(new GameStore())
