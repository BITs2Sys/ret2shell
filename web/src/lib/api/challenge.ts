import type { Challenge, Tag } from '$lib/models/challenge'
import { api, api_root } from '.'

export async function getChallengeList(game_id: number, page: number, per_page: number) {
  let uri = `${api_root}/challenge?page=${page}&per_page=${per_page}&game_id=${game_id}`
  const response = await api.get(uri)
  return response.data as { challenges: Challenge[]; total: number }
}

export async function getTagList() {
  return (await api.get(`${api_root}/challenge/tag`)).data as Tag[]
}

export async function getChallenge(id: number) {
  return (await api.get(`${api_root}/challenge/${id}`)).data as Challenge
}
