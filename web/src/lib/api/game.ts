import type { Article } from "@models/article";
import type { Challenge, ChallengeEnv } from "@models/challenge";
import type { Game, HostType } from "@models/game";
import type { Instance } from "@models/instance";
import type { Submission } from "@models/submission";
import { type Team, TeamState } from "@models/team";
import type { User } from "@models/user";
import type { Pod } from "kubernetes-types/core/v1";
import type { SearchParamsOption } from "ky";
import type { DateTime } from "luxon";
import api, { api_root } from ".";
import type { Extra } from "../models/extra";
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

export async function publishChallenge(game_id: number, challenge_id: number) {
  return await api.post(`${api_root}/game/${game_id}/challenge/${challenge_id}/publish`).json<Challenge>();
}

export async function withdrawChallenge(game_id: number, challenge_id: number) {
  return await api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/publish`).json<Challenge>();
}

export async function deleteChallenge(game_id: number, challenge_id: number) {
  return await api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}`).json<void>();
}

export async function getChallengeHint(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/hint`).json<Hint[]>();
}

export async function unlockChallengeHint(game_id: number, challenge_id: number, hint_id: number) {
  return await api
    .post(`${api_root}/game/${game_id}/challenge/${challenge_id}/hint/unlock`, {
      json: {
        id: hint_id,
      },
    })
    .json<Extra>();
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

export async function getChallengeAttachments(
  game_id: number,
  challenge_id: number,
  all?: boolean,
  folder?: "static" | "mapped" | "checker"
) {
  return await api
    .get(`${api_root}/game/${game_id}/challenge/${challenge_id}/file`, {
      searchParams: JSON.parse(
        JSON.stringify({
          all,
          folder,
        })
      ) as SearchParamsOption,
    })
    .json<{ folder: "static" | "mapped" | "checker"; file: string }[]>();
}

export async function deleteChallengeAttachment(
  game_id: number,
  challenge_id: number,
  folder: "static" | "mapped" | "checker",
  file: string
) {
  return await api
    .delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/file`, {
      searchParams: {
        folder,
        file,
      },
    })
    .json<void>();
}

export async function getChallengeEnv(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/env`).json<ChallengeEnv | null>();
}

export async function getChallengeInstance(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/instance`).json<Pod[]>();
}

export async function updateChallengeEnv(game_id: number, challenge_id: number, env: ChallengeEnv) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/env`, {
      json: env,
    })
    .json<void>();
}

export async function deleteChallengeEnv(game_id: number, challenge_id: number) {
  return await api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/env`).json<void>();
}

export async function getChallengeCheckerScript(game_id: number, challenge_id: number, lint?: boolean) {
  return await api
    .get(`${api_root}/game/${game_id}/challenge/${challenge_id}/checker`, {
      searchParams: JSON.parse(
        JSON.stringify({
          lint,
        })
      ),
    })
    .json<{
      script: string;
      lint?: string;
    }>();
}

export async function updateChallengeCheckerScript(game_id: number, challenge_id: number, content: string) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/checker`, {
      json: {
        content,
      },
    })
    .json<void>();
}

export async function getTeamInfo(game_id: number, team_id: number, ex?: boolean) {
  return await api
    .get(`${api_root}/game/${game_id}/team/${team_id}`, {
      searchParams: JSON.parse(
        JSON.stringify({
          ex,
        })
      ),
    })
    .json<Team>();
}

export async function getTeamMembers(game_id: number, team_id: number) {
  return await api.get(`${api_root}/game/${game_id}/team/${team_id}/member`).json<User[]>();
}

export async function getSelfTeam(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/team/self`).json<Team>();
}

export async function updateSelfteam(
  game_id: number,
  team: {
    name: string;
    institute_id: number | null;
  }
) {
  return await api.patch(`${api_root}/game/${game_id}/team/self`, { json: team }).json<Team>();
}

export async function getTeamExtras(game_id: number, team_id: number) {
  return await api.get(`${api_root}/game/${game_id}/team/${team_id}/extra`).json<Extra[]>();
}

export async function getTeamSolves(game_id: number, team_id: number) {
  return await api.get(`${api_root}/game/${game_id}/team/${team_id}/solve`).json<Submission[]>();
}

export async function createTeam(
  game_id: number,
  team: {
    name: string;
  }
) {
  return await api.post(`${api_root}/game/${game_id}/team`, { json: team }).json<Team>();
}

export async function joinTeam(game_id: number, token: string) {
  return await api
    .patch(`${api_root}/game/${game_id}/team`, {
      json: {
        token,
      },
    })
    .json<Team>();
}

export type EventDeviceInfo = {
  client: string;
  address: string;
  connected_at: DateTime;
};

export async function getGameDevices(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/device`).json<EventDeviceInfo[]>();
}

export async function getGameAdmins(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/administrator`).json<User[]>();
}

export async function updateGameAdmins(game_id: number, admins: number[]) {
  return await api.patch(`${api_root}/game/${game_id}/administrator`, { json: admins }).json<Game>();
}

export async function getGameSelfEnvs(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/env`).json<Instance[]>();
}

export async function delayGameSelfEnv(game_id: number) {
  return await api.patch(`${api_root}/game/${game_id}/env`).json<void>();
}

export async function stopGameSelfEnv(game_id: number) {
  return await api.delete(`${api_root}/game/${game_id}/env`).json<void>();
}

export async function startChallengeEnv(game_id: number, challenge_id: number) {
  return await api.post(`${api_root}/game/${game_id}/challenge/${challenge_id}/env`).json<void>();
}

export async function submitFlag(game_id: number, challenge_id: number, flag: string) {
  return await api
    .post(`${api_root}/game/${game_id}/challenge/${challenge_id}/submit`, {
      json: {
        content: flag,
      },
    })
    .json<Submission>();
}

export async function checkSubmissionStatus(game_id: number, challenge_id: number, submission_id: number) {
  return await api
    .get(`${api_root}/game/${game_id}/challenge/${challenge_id}/submit`, {
      searchParams: {
        id: submission_id,
      },
    })
    .json<Submission>();
}

export async function getSelfSolves(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/solve`).json<Submission[]>();
}

export async function getChallengeSolveStatus(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/submit`).json<{
    solved: boolean;
    solves: number;
    top: Submission[];
  }>();
}
