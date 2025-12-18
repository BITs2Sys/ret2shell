import { useChallenges } from "@api/challenge";
import { useGame } from "@api/game";
import { useSelfTeam, useTeamRank } from "@api/team";
import { isAdminOfGame, isGameInArchived } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Link from "@widgets/link";
import Progress from "@widgets/progress";
import { createMemo, Match, Switch } from "solid-js";

export default function (props: { gameId: number }) {
  const game = useGame({ id: () => props.gameId });
  const team = useSelfTeam({ game_id: () => props.gameId, silenced: true });
  const rank = useTeamRank({
    game_id: () => props.gameId,
    team_id: () => team.data?.id || 0,
    enabled: () => !!team.data?.id,
  });
  const challenges = useChallenges({
    game_id: () => props.gameId,
    enabled: () => !!game.data,
  });

  const solvedChallenges = createMemo(() => team.data?.history.filter((h) => !!h.challenge_id).length ?? 0);
  const totalChallenges = createMemo(() => challenges.data?.[0].length ?? 0);
  return (
    <div class="border-b border-b-layer-content/10 px-2 h-16 shrink-0 flex items-center justify-center relative">
      <Switch>
        <Match when={isAdminOfGame(game.data)}>
          <Button ghost disabled class="w-full" justify="start">
            <span class="shrink-0 icon-[fluent--person-settings-20-filled] w-5 h-5 text-error" />
            <span>{t("game.adminMode")}</span>
          </Button>
        </Match>
        <Match when={team.data}>
          <Link ghost class="w-full" justify="start" href={`/games/${props.gameId}/teams/${team.data?.id}`}>
            <span class="shrink-0 icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
            <span class="flex-1 text-start truncate">{team.data?.name}</span>
            <span class="text-success">{team.data?.score} pts</span>
            <span class="text-warning">#{rank.data ?? "-"}</span>
          </Link>
          <Progress
            class="absolute bottom-2 left-4 right-4"
            max={1}
            min={0}
            value={(solvedChallenges() ?? 0) / (totalChallenges() || 1)}
            static
          />
        </Match>
        <Match when={isGameInArchived(game.data)}>
          <Button ghost disabled class="w-full" justify="start">
            <span class="shrink-0 icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
            <span>{t("game.ended")}</span>
          </Button>
        </Match>
        <Match when={true}>
          <Button ghost disabled class="w-full" justify="start">
            <span class="shrink-0 icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
            <span>{t("game.canNotParticipate")}</span>
          </Button>
        </Match>
      </Switch>
    </div>
  );
}
