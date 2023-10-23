export interface ScoreHistory {
  score: number
  time: number
  challenge_id: number | null
  blood_state: number | null
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

export interface TeamWithGameName {
  id: number
  name: string
  game_id: number
  game_name: string
  state: State
  institute_id: number | null
  score: number
  last_active_at: number
}

export interface TeamList {
  teams: Team[]
  total: number
}

export interface TeamRank {
  rank: number
}
