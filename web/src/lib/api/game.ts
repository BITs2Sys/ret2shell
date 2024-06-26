import type { Article } from "@models/article";
import type { Challenge } from "@models/challenge";
import type { Game, HostType } from "@models/game";
import { type Team, TeamState } from "@models/team";
import type { SearchParamsOption } from "ky";
import api, { api_root } from ".";
import type { Hint } from "../models/hint";

export async function getGames(page?: number, page_size?: number, host_type?: HostType, weight?: number) {
    return (
        await api.get(`${api_root}/game`, {
            searchParams: JSON.parse(
                JSON.stringify({
                    page,
                    page_size,
                    host_type,
                    weight,
                })
            ) as SearchParamsOption,
        })
    ).json<[Game[], number]>();
}

export async function getGame(id: number) {
    return await api.get(`${api_root}/game/${id}`).json<Game>();
}

export async function createGame(game: Game) {
    return await api.post(`${api_root}/game`, { json: game }).json<Game>();
}

export async function updateGame(id: number, game: Game) {
    return await api.patch(`${api_root}/game/${id}`, { json: game }).json<Game>();
}

export async function deleteGame(id: number) {
    return await api.delete(`${api_root}/game/${id}`).json<null>();
}

export async function getGameIntroduction(id: number) {
    return await api.get(`${api_root}/game/${id}/introduction`).json<Article>();
}

export async function updateGameIntroduction(id: number, article: Article) {
    return await api.patch(`${api_root}/game/${id}/introduction`, { json: article }).json<Article>();
}

export async function getGameScoreboard(
    id: number,
    page?: number,
    page_size?: number,
    with_hidden?: boolean,
    institute_id?: number
) {
    return (
        await api.get(`${api_root}/game/${id}/team`, {
            searchParams: JSON.parse(
                JSON.stringify({
                    min_state: with_hidden ? TeamState.Hidden : TeamState.Passed,
                    page,
                    page_size,
                    institute_id,
                    asc: false,
                    order_by: "score",
                })
            ) as SearchParamsOption,
        })
    ).json<[Team[], number]>();
}

export async function getChallengeList(game_id: number, page?: number, page_size?: number) {
    return (
        await api.get(`${api_root}/game/${game_id}/challenge`, {
            searchParams: JSON.parse(
                JSON.stringify({
                    page,
                    page_size,
                })
            ) as SearchParamsOption,
        })
    ).json<[Challenge[], number]>();
}

export async function getChallenge(game_id: number, challenge_id: number) {
    return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}`).json<Challenge>();
}

export async function createChallenge(game_id: number, challenge: Challenge) {
    return await api.post(`${api_root}/game/${game_id}/challenge`, { json: challenge }).json<Challenge>();
}

export async function updateChallenge(game_id: number, challenge: Challenge) {
    return await api
        .patch(`${api_root}/game/${game_id}/challenge/${challenge.id}`, { json: challenge })
        .json<Challenge>();
}

export async function deleteChallenge(game_id: number, challenge_id: number) {
    return await api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}`).json<void>();
}

export async function getChallengeHint(game_id: number, challenge_id: number) {
    return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/hint`).json<Hint[]>();
}

export async function createChallengeHint(game_id: number, challenge_id: number, hint: Hint) {
    return await api
        .post(`${api_root}/game/${game_id}/challenge/${challenge_id}/hint`, {
            json: hint,
        })
        .json<Hint[]>();
}

export async function deleteChallengeHint(game_id: number, challenge_id: number, hint_id: number) {
    return await api
        .delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/hint`, {
            searchParams: {
                id: hint_id,
            },
        })
        .json<void>();
}
