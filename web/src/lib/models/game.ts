export interface Game {
  id: number
  updated_at: number
  name: string
  brief: string
  introduction: string
  start_time: number
  end_time: number
  register_time: number
  archive_time: number
  hidden: boolean
  frozen: boolean
  host_as_game: boolean
  team_size_limit: number
  cover_path: string | null
  enable_team_audit: boolean
  can_register_after_started: boolean
  institute_id: number | null
  blood_award_rate: number
}

export interface Notification {
  id: number
  title: string
  content: string
  published_at: number
  game_id: number
}
