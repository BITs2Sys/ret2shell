import type { Game } from "@models/game";
import type { Team } from "@models/team";
import { Permission, type User } from "@models/user";
import { DateTime } from "luxon";
import { createStore } from "solid-js/store";
import { accountStore } from "./account";
import { t } from "./theme";

export const [gameStore, setGameStore] = createStore({
  games: [] as Game[],
  current: null as Game | null,
  preload: null as Game | null,
  team: null as Team | null,
  rank: null as number | null,
  score: null as number | null,
  members: [] as User[],
  showTeamCover: false,
});

export type GameStoreType = typeof gameStore;

export function appendGames(games: Game[]) {
  const ids = new Set(games.map((g) => g.id));
  setGameStore({
    games: [...gameStore.games.filter((g) => !ids.has(g.id)), ...games],
  });
}

export function inProgress() {
  if (
    gameStore.current?.start_at &&
    gameStore.current?.start_at < DateTime.now() &&
    gameStore.current?.end_at &&
    gameStore.current?.end_at > DateTime.now()
  ) {
    return true;
  }
  return false;
}

export function inRegister() {
  const register_end = gameStore.current?.can_register_after_started
    ? gameStore.current?.end_at
    : gameStore.current?.start_at;
  if (
    gameStore.current?.register_at &&
    gameStore.current?.register_at < DateTime.now() &&
    register_end &&
    register_end > DateTime.now()
  ) {
    return true;
  }
  return false;
}

export function inArchiving() {
  if (
    gameStore.current?.end_at &&
    gameStore.current?.end_at < DateTime.now() &&
    gameStore.current?.archive_at &&
    gameStore.current?.archive_at > DateTime.now()
  ) {
    return true;
  }
  return false;
}

export function inArchived() {
  if (gameStore.current?.archive_at && gameStore.current.archive_at < DateTime.now()) {
    return true;
  }
  return false;
}

export const canParticipate = () => {
  if (
    !gameStore.current?.can_register_after_started &&
    gameStore.current?.start_at &&
    gameStore.current.start_at < DateTime.now()
  ) {
    return false;
  }
  if (gameStore.current?.end_at && gameStore.current.end_at < DateTime.now()) {
    return false;
  }
  if (isGameAdmin()) {
    return false;
  }
  if (accountStore.permissions.includes(Permission.Host)) {
    return false;
  }
  if (gameStore.current?.access_policy.restrict) {
    if (
      accountStore.info?.institute_id &&
      gameStore.current.access_policy.institutes.includes(accountStore.info.institute_id)
    )
      return true;
    return false;
  }

  return true;
};

export function canAccessChallenges(): [boolean, string] {
  if (!accountStore.id) return [false, t("game.team.loginThenBack")!];
  if (isGameAdmin()) {
    return [true, ""];
  }
  if (gameStore.current?.start_at && gameStore.current.start_at > DateTime.now()) {
    return [false, t("game.challenge.notStarted")!];
  }
  if (inArchived()) {
    return [false, t("game.ended")!];
  }
  if (inProgress() || inArchiving()) {
    if (gameStore.team) {
      return [true, ""];
    }
  }
  return [false, t("game.team.joinFirst")!];
}

export function isGameAdmin() {
  if (!accountStore.id) return false;
  if (gameStore.current?.admins.includes(accountStore.id) && accountStore.permissions.includes(Permission.Game)) {
    return true;
  }
  return false;
}

export function gameParticipateState() {
  if (gameStore.current?.register_at && gameStore.current.register_at > DateTime.now()) {
    return [false, t("game.registerNotStarted")!];
  }
  if (!inRegister()) {
    return [false, t("game.registerEnded")!];
  }
  return [true, ""];
}
