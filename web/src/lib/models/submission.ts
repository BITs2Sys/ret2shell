export interface Submission {
  id: number
  created_at: number
  user_id: number
  challenge_id: number
  content: string
  solved: boolean
  with_score: boolean
}

export interface SubmissionOnlyUserInfo {
  id: number
  created_at: number
  user_id: number
  user_name: string
}