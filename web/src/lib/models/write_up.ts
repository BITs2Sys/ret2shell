export interface WriteUp {
  id: number
  title: string
  published_at: number
  updated_at: number
  author_id: number
  team_id: number
  game_id: number
  content: string
  hidden: boolean
}

export interface WriteUpOnlyTeamInfo {
  id: number
  title: string
  team_id: number
  team_name: string
  hidden: boolean
}

export interface WriteUpWithInfo {
  id: number
  title: string
  published_at: number
  updated_at: number
  author_id: number
  author_name: string
  team_id: number
  team_name: string
  game_id: number
  content: string
  hidden: boolean
}
