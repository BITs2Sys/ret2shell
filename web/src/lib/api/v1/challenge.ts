import type { Answer } from '$lib/models/answer'
import type { Challenge, Tag } from '$lib/models/challenge'
import type { Hint } from '$lib/models/hint'
import type {
  Submission,
  SubmissionOnlyTeamInfo,
  SubmissionOnlyUserInfo,
  SubmissionWithInfo,
} from '$lib/models/submission'
import { api, api_root } from '..'

export async function getChallengeList(
  game_id: number,
  page?: number,
  per_page?: number,
  tag_id: number | null = null
) {
  let uri = `${api_root}/v1/challenge?game_id=${game_id}`
  if (page && per_page) {
    uri += `&page=${page}&per_page=${per_page}`
  }
  if (tag_id !== undefined && tag_id !== null) {
    uri += `&tag_id=${tag_id}`
  }
  const response = await api.get(uri)
  return response.data as { challenges: Challenge[]; total: number }
}

export async function getTagList() {
  return (await api.get(`${api_root}/v1/challenge/tag`)).data as Tag[]
}

export async function createTag(tag: Tag) {
  return await api.post(`${api_root}/v1/challenge/tag`, tag)
}

export async function deleteTag(id: number) {
  return await api.delete(`${api_root}/v1/challenge/tag/${id}`)
}

export async function createChallenge(game_id: number, challenge: Challenge) {
  return (await api.post(`${api_root}/v1/challenge?game_id=${game_id}`, challenge)).data as Challenge
}

export async function updateChallenge(id: number, challenge: Challenge) {
  return (await api.patch(`${api_root}/v1/challenge/${id}`, challenge)).data as Challenge
}

export async function deleteChallenge(id: number) {
  return await api.delete(`${api_root}/v1/challenge/${id}`)
}

export async function getChallenge(id: number) {
  return (await api.get(`${api_root}/v1/challenge/${id}`)).data as Challenge
}

export async function getChallengeHints(id: number) {
  return (await api.get(`${api_root}/v1/challenge/${id}/hint`)).data as Hint[]
}

export async function createHint(challenge_id: number, hint: Hint) {
  return (await api.post(`${api_root}/v1/challenge/${challenge_id}/hint`, hint)).data as Hint
}

export async function deleteHint(challenge_id: number, id: number) {
  return await api.delete(`${api_root}/v1/challenge/${challenge_id}/hint?hint_id=${id}`)
}

export async function submitFlag(submission: Submission) {
  return (await api.post(`${api_root}/v1/challenge/${submission.challenge_id}/submission`, submission)).data as {
    result: boolean
    blood_state: number
  }
}

export async function downloadChallengeAttachment(
  game_id: number,
  challenge_id: number,
  file: string,
  callback?: (progress: number) => void
) {
  const resp = await api.get(`${api_root}/v1/game/${game_id}/challenge/${challenge_id}/attachment?file=${file}`, {
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

export async function getChallengeSubmission(id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/v1/challenge/${id}/submission`
  if (page && per_page) {
    uri += `?page=${page}&per_page=${per_page}`
  }
  return (await api.get(uri)).data as { submissions: SubmissionWithInfo[]; total: number }
}

export async function getChallengeSolvedUser(id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/v1/challenge/${id}/solved-user`
  if (page && per_page) {
    uri += `?page=${page}&per_page=${per_page}`
  }
  return (await api.get(uri)).data as { users: SubmissionOnlyUserInfo[]; total: number }
}

export async function getChallengeSolvedTeam(id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/v1/challenge/${id}/solved-team`
  if (page && per_page) {
    uri += `?page=${page}&per_page=${per_page}`
  }
  return (await api.get(uri)).data as { teams: SubmissionOnlyTeamInfo[]; total: number }
}

export async function getChallengeAnwser(id: number) {
  return (await api.get(`${api_root}/v1/challenge/${id}/answer`)).data as Answer
}

export async function getChallengeStatistics(id: number) {
  return (await api.get(`${api_root}/v1/challenge/${id}/statistics`)).data as {
    submissions_count: number
    solves_count: number
    instances_count: number
    running_instances_count: number
  }
}

export async function getChallengeAnswer(challenge_id: number) {
  return (await api.get(`${api_root}/v1/challenge/${challenge_id}/answer`)).data as Answer
}

export async function createChallengeAnswer(challenge_id: number, answer: Answer) {
  return (await api.post(`${api_root}/v1/challenge/${challenge_id}/answer`, answer)).data as Answer
}

export async function updateChallengeAnswer(challenge_id: number, answer: Answer) {
  return (await api.patch(`${api_root}/v1/challenge/${challenge_id}/answer`, answer)).data as Answer
}

export async function deleteChallengeAnswer(challenge_id: number) {
  return await api.delete(`${api_root}/v1/challenge/${challenge_id}/answer`)
}
