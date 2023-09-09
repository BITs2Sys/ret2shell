export interface ScoreHistory {
  score: number
  time: number
}

export enum State {
  Banned = 0,
  NeedAudit = 1,
  Normal = 2,
  Hidden = 3,
}

export interface Team {
  id: number
  name: string
  game_id: number
  token: string
  state: State
  institute_id: number | null
  score: number
  history: ScoreHistory[]
  last_active_at: number
}

export interface TeamList {
  teams: Team[]
  total: number
}

export interface TeamRank {
  rank: number
}
