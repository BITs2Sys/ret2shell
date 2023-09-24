import type { Challenge, Tag } from '$lib/models/challenge'
import type { Hint } from '$lib/models/hint'
import type { Submission } from '$lib/models/submission'
import { api, api_root } from '.'

export async function getChallengeList(game_id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/challenge?game_id=${game_id}`
  if (page && per_page) {
    uri += `&page=${page}&per_page=${per_page}`
  }
  const response = await api.get(uri)
  return response.data as { challenges: Challenge[]; total: number }
}

export async function getTagList() {
  return (await api.get(`${api_root}/challenge/tag`)).data as Tag[]
}

export async function getChallenge(id: number) {
  return (await api.get(`${api_root}/challenge/${id}`)).data as Challenge
}

export async function getChallengeHints(id: number) {
  return (await api.get(`${api_root}/challenge/${id}/hint`)).data as Hint[]
}

export async function submitFlag(submission: Submission) {
  return await api.post(`${api_root}/challenge/${submission.challenge_id}/submission`, submission)
}
