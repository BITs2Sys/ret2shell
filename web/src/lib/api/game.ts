import type { Game, Notification } from '$lib/models/game'
import type { Submission, SubmissionWithInfo } from '$lib/models/submission'
import type { Team } from '$lib/models/team'
import type { User } from '$lib/models/user'
import type { WriteUp, WriteUpOnlyTeamInfo, WriteUpWithInfo } from '$lib/models/write_up'
import { api, api_root } from '.'

export async function getGameList(page: number, per_page: number, host_as_game?: boolean) {
  let uri = `${api_root}/game?page=${page}&per_page=${per_page}`
  if (host_as_game !== undefined) uri += `&host_as_game=${host_as_game}`
  const response = await api.get(uri)
  return response.data as { games: Game[]; total: number }
}

export async function getGame(id: number) {
  return (await api.get(`${api_root}/game/${id}`)).data as Game
}

export async function createGame(game: Game) {
  return await api.post(`${api_root}/game`, game)
}

export async function updateGame(id: number, game: Game) {
  return await api.patch(`${api_root}/game/${id}`, game)
}

export interface CreateTeamRequest {
  name: string
  captcha_id: string
  captcha_answer: string
}

export async function createTeam(game_id: number, req: CreateTeamRequest) {
  return await api.post(`${api_root}/game/${game_id}/team`, req)
}

export interface JoinTeamRequest {
  token: number
  captcha_id: string
  captcha_answer: string
}

export async function joinTeam(game_id: number, req: JoinTeamRequest) {
  return await api.patch(`${api_root}/game/${game_id}/team`, req)
}

export async function getGameSubmission(game_id: number, page: number, per_page: number) {
  return (await api.get(`${api_root}/game/${game_id}/submission?page=${page}&per_page=${per_page}`)).data as {
    submissions: SubmissionWithInfo[]
    total: number
  }
}

export async function getGameSelfSolves(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/solved`)).data as Submission[]
}

export async function getGameTeamSolves(game_id: number, team_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/solved?team_id=${team_id}`)).data as Submission[]
}

export async function getGameTeamList(
  game_id: number,
  page: number,
  per_page: number,
  filter?: string,
  filterNeedAudit?: boolean
) {
  let uri = `${api_root}/game/${game_id}/team?page=${page}&per_page=${per_page}`
  if (filter !== undefined && filter !== null && filter !== '') uri += `&filter=${filter}`
  if (filterNeedAudit !== undefined && filterNeedAudit !== null) uri += `&need_audit=${filterNeedAudit}`
  // console.log(uri)
  return (await api.get(uri)).data as { teams: Team[]; total: number }
}

export async function getTeamInfo(game_id: number, team_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/team/info?team_id=${team_id}`)).data as Team
}

export async function getTeamMembers(game_id: number, team_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/team/info/members?team_id=${team_id}`)).data as User[]
}

export async function getSelfTeamInfo(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/team/self`)).data as Team
}

export async function getSelfTeamRank(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/team/self/rank`)).data as { rank: number }
}

export async function getSelfTeamMembers(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/team/self/members`)).data as User[]
}

export async function updateTeamInfo(game_id: number, team_id: number, team: Team) {
  return await api.patch(`${api_root}/game/${game_id}/team/info?team_id=${team_id}`, team)
}

export async function changeTeamAudit(game_id: number, team_id: number) {
  return await api.patch(`${api_root}/game/${game_id}/team/audit?team_id=${team_id}`)
}

export async function getGameNotifications(game_id: number, page: number, per_page: number) {
  return (await api.get(`${api_root}/game/${game_id}/notification?page=${page}&per_page=${per_page}`)).data as {
    notifications: Notification[]
    total: number
  }
}

export async function createNotification(game_id: number, notification: Notification) {
  return await api.post(`${api_root}/game/${game_id}/notification`, notification)
}

export async function deleteGameNotification(game_id: number, notification_id: number) {
  return await api.delete(`${api_root}/game/${game_id}/notification?notification_id=${notification_id}`)
}

export interface Scoreboard {
  teams: Team[]
  total: number
}

export async function getGameScoreboard(
  game_id: number,
  page: number,
  per_page: number,
  all?: boolean,
  institute?: number | null
) {
  let uri = `${api_root}/game/${game_id}/scoreboard?page=${page}&per_page=${per_page}`
  if (all !== undefined && all !== null) uri += `&all=${all}`
  if (institute !== undefined && institute !== null) uri += `&institute=${institute}`
  return (await api.get(uri)).data as Scoreboard
}

export async function getGameWriteUp(game_id: number, page?: number, per_page?: number) {
  let uri = `${api_root}/game/${game_id}/writeup`
  if (page && per_page) {
    uri += `?page=${page}&per_page=${per_page}`
  }
  return (await api.get(uri)).data as { writeups: WriteUpOnlyTeamInfo[]; total: number }
}

export async function getGameTeamWriteUp(game_id: number, id: number) {
  return (await api.get(`${api_root}/game/${game_id}/writeup/detail?writeup_id=${id}`)).data as WriteUpWithInfo
}

export async function auditGameTeamWriteUp(game_id: number, id: number, audit: boolean) {
  return await api.patch(`${api_root}/game/${game_id}/writeup/audit?writeup_id=${id}&audit=${audit}`)
}

export async function getGameTeamWriteUpSelf(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/writeup/self`)).data as WriteUp
}

export async function updateGameTeamWriteUpSelf(game_id: number, writeup: WriteUp) {
  return await api.patch(`${api_root}/game/${game_id}/writeup/self`, writeup)
}

export async function createGameTeamWriteUpSelf(game_id: number, writeup: WriteUp) {
  return await api.post(`${api_root}/game/${game_id}/writeup`, writeup)
}
