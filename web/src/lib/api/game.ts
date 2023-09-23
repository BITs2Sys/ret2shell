import type { Game } from '$lib/models/game'
import type { Submission } from '$lib/models/submission'
import type { Team } from '$lib/models/team'
import { api, api_root } from '.'

export async function getGameList(page: number, per_page: number, host_as_game: boolean) {
  const uri = `${api_root}/game?page=${page}&per_page=${per_page}&host_as_game=${host_as_game}`
  const response = await api.get(uri)
  return response.data as { games: Game[]; total: number }
}

export async function getGame(id: number) {
  return (await api.get(`${api_root}/game/${id}`)).data as Game
}

export async function getGameSelfSubmission(game_id: number) {
  return (await api.get(`${api_root}/game/${game_id}/solved`)).data as Submission[]
}

export async function getGameTeamSubmission(game_id: number, team_id: number) {
  return (await api.get(`${api_root}/team/${game_id}/solved?team_id=${team_id}`)).data as Submission[]
}

export async function getTeamInfo(game_id: number, team_id: number) {
  return (await api.get(`${api_root}/team/${game_id}/team/info?team_id=${team_id}`)).data as Team
}

export async function getSelfTeamInfo(game_id: number) {
  return (await api.get(`${api_root}/team/${game_id}/team/self`)).data as Team
}

export async function getGameNotifications(game_id: number, page: number, per_page: number) {
  return (await api.get(`${api_root}/game/${game_id}/notification?page=${page}&per_page=${per_page}`)).data as {
    notifications: Notification[]
    total: number
  }
}
