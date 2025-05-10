import { gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";

import GitBlock from "@blocks/game/git";

export default function () {
  return (
    <>
      <Title page={t("game.git.title")} route={`/games/${gameStore.current?.id}/admin/git`} />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center">
        <GitBlock />
      </div>
    </>
  );
}
