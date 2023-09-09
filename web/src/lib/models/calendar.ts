export interface Calendar {
  id: number
  name: string
  intro: string
  link: string
  start_time: number
  end_time: number
  audited: boolean
  game_id: number | null
  reporter_id: number
}
