import type { Challenge, Tag } from '$lib/models/challenge'
import type { Hint } from '$lib/models/hint'
import type { Submission, SubmissionOnlyUserInfo } from '$lib/models/submission'
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

export async function downloadChallengeAttachment(
  game_id: number,
  challenge_id: number,
  file: string,
  callback?: (progress: number) => void
) {
  const resp = await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/attachment?file=${file}`, {
    responseType: 'blob',
    onDownloadProgress: (e) => {
      let progress = Math.floor((e.loaded * 100) / (e.total || -1))
      if (progress < 0) progress = 0
      if (callback) callback(progress)
    },
  })
  const url = window.URL.createObjectURL(new Blob([resp.data]))
  const link = document.createElement('a')
  link.href = url
  link.setAttribute('download', file)
  document.body.appendChild(link)
  link.click()
  link.remove()
}

export async function getChallengeSolvedUser(id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/challenge/${id}/solved`
  if (page && per_page) {
    uri += `?page=${page}&per_page=${per_page}`
  }
  return (await api.get(uri)).data as { records: SubmissionOnlyUserInfo[]; total: number }
}
