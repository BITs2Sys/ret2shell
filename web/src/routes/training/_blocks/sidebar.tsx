import { useGame } from "@api/game";
import ChallengeList from "@blocks/challenge/list";
import { HostType } from "@models/game";
import { useNavigate, useParams } from "@solidjs/router";
import { t } from "@storage/theme";
import Link from "@widgets/link";
import { HTTPError } from "ky";
import { createMemo, Show } from "solid-js";
import Playgrounds from "./playgrounds";

export default function SideBar() {
  const params = useParams();
  const navigate = useNavigate();

  const selectedGameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || 0);
  const game = useGame({
    id: () => selectedGameId(),
    enabled: () => selectedGameId() > 0,
    onError: (err) => {
      if (err instanceof HTTPError) navigate(`/sigtrap/${err.response.status}`, { replace: true });
      else navigate("/sigtrap/unknown", { replace: true });
      return false;
    },
  });

  return (
    <div class="flex flex-col overflow-hidden w-full h-full">
      <div class="border-b border-b-layer-content/10 px-2 h-16 flex space-x-2 items-center">
        <Show
          when={game.data}
          fallback={
            <Link class="flex-1" ghost justify="start" href="/training">
              <span class="shrink-0 icon-[fluent--dumbbell-20-filled] w-5 h-5 text-primary" />
              <span>{t("training.list")}</span>
            </Link>
          }
        >
          <Link class="flex-1" ghost justify="start" href={`/training/${game.data?.id}`}>
            <span class="shrink-0 icon-[fluent--dumbbell-20-filled] w-5 h-5 text-primary" />
            <span class="flex-1 text-start">{game.data?.name}</span>
            <Show when={game.data?.hidden}>
              <span class="shrink-0 icon-[fluent--eye-off-20-regular] w-5 h-5 text-warning" />
            </Show>
          </Link>
          <Show when={game.data?.host_type === HostType.Game}>
            <Link square ghost href={`/games/${game.data?.id}`} title={t("training.gotoGamePage")}>
              <span class="shrink-0 icon-[fluent--info-20-regular] w-5 h-5 text-" />
            </Link>
          </Show>
          <Link square ghost href="/training" title={t("general.actions.back.title")}>
            <span class="shrink-0 icon-[fluent--arrow-hook-up-left-20-regular] w-5 h-5 text-warning" />
          </Link>
        </Show>
      </div>
      <Show when={selectedGameId() > 0} fallback={<Playgrounds />}>
        <ChallengeList training gameId={selectedGameId()} challengeId={0} />
      </Show>
    </div>
  );
}
