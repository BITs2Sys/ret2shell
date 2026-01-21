import GameStatistics from "@blocks/game/statistics";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { createMemo } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  return (
    <>
      <Title page={t("game.statistics.title")} route={`/games/${gameId()}/admin/statistics`} />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center">
        <GameStatistics gameId={gameId()} />
      </div>
    </>
  );
}
