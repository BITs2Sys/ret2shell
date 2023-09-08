export interface Action {
  id: number
  created_at: number
  started_at: number | null
  challenge_id: number
  status: number
  commit_id: string
}
