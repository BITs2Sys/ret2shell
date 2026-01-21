import { useAccountProfile } from "@api/account";
import type { Game } from "@models/game";
import { type Team, TeamState } from "@models/team";
import { Permission } from "@models/user";
import { DateTime } from "luxon";
import { accountStore } from "./account";
import { t } from "./theme";

// export async function refreshSelfTeam() {
//   if (gameStore.current && !isGameAdmin() && accountStore.permissions.includes(Permission.Verified)) {
//     try {
//       const team = await getSelfTeam(gameStore.current?.id);
//       setGameStore({ team });
//       if (!team) return;
//       try {
//         const rank = await getTeamRank(gameStore.current!.id, team.id);
//         setGameStore({ rank });
//       } catch {
//         setGameStore({ rank: null });
//       }
//       if (team.state === TeamState.Pending) {
//         addToast({
//           level: "warning",
//           description: t("team.status.pending.message"),
//           duration: 5000,
//         });
//       }
//     } catch {
//       setGameStore({ team: null });
//     }
//   }
// }

export function isGameInProgress(game?: Game) {
  if (game?.start_at && game.start_at < DateTime.now() && game?.end_at && game.end_at > DateTime.now()) {
    return true;
  }
  return false;
}

export function isGameInRegister(game?: Game) {
  const register_end = game?.can_register_after_started ? game.end_at : game?.start_at;
  if (game?.register_at && game.register_at < DateTime.now() && register_end && register_end > DateTime.now()) {
    return true;
  }
  return false;
}

export function isGameInArchiving(game?: Game) {
  if (game?.end_at && game.end_at < DateTime.now() && game?.archive_at && game.archive_at > DateTime.now()) {
    return true;
  }
  return false;
}

export function isGameInArchived(game?: Game) {
  if (game?.archive_at && game.archive_at < DateTime.now()) {
    return true;
  }
  return false;
}

export function isGameCanParticipate(game?: Game) {
  const account = useAccountProfile();
  if (!game) return false;
  if (!game.can_register_after_started && game.start_at && game.start_at < DateTime.now()) {
    return false;
  }
  if (game.end_at && game.end_at < DateTime.now()) {
    return false;
  }
  if (isAdminOfGame(game)) {
    return false;
  }
  if (game.access_policy.restrict) {
    const instituteId = account.data?.institute_id;
    if (instituteId && game.access_policy.institutes.includes(instituteId)) return true;
    return false;
  }

  return true;
}

export function isPlayerCanAccessChallenges(game?: Game, team?: Team | null): [boolean, string] {
  if (!accountStore.id) return [false, t("general.network.status.401.title")];
  if (isAdminOfGame(game)) {
    return [true, ""];
  }
  if (game?.start_at && game.start_at > DateTime.now()) {
    return [false, t("game.notStarted")];
  }
  if (isGameInArchived(game)) {
    return [true, t("game.ended")];
  }
  if (isGameInProgress(game) || isGameInArchiving(game)) {
    if (team) {
      return [true, ""];
    }
  }
  if (team?.state === TeamState.Pending) {
    return [false, t("team.status.pending.message")];
  }
  if (team?.state === TeamState.Banned) {
    return [false, t("team.status.banned.message")];
  }
  return [false, t("team.status.empty.message")];
}

export function isAdminOfGame(game?: Game) {
  if (!accountStore.id) return false;
  if (game?.admins.includes(accountStore.id) && accountStore.permissions.includes(Permission.Game)) {
    return true;
  }
  return false;
}

export function gameParticipateState(game: Game): [boolean, string] {
  if (game.register_at && game.register_at > DateTime.now()) {
    return [false, t("game.registerNotStarted")];
  }
  if (!isGameInRegister(game)) {
    return [false, t("game.registerEnded")];
  }
  return [true, ""];
}

// find the last one
export function currentTimelinePeriod(game?: Game) {
  const len = game?.timeline_presets?.length;
  const sortedTimeline = game?.timeline_presets?.sort((a, b) => a.start_at.toMillis() - b.start_at.toMillis());
  if (!len) return null;
  for (let i = len; i > 0; i--) {
    const period = sortedTimeline![i - 1];
    if (period.start_at < DateTime.now() && period.end_at > DateTime.now()) {
      return period;
    }
  }
}
