import { gameStore, isGameAdmin } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import { Match, Switch } from "solid-js";

export default function () {
  return (
    <>
      <div class="border-b border-b-layer-content/10 px-2 h-16 flex-shrink-0 flex items-center justify-center">
        <Switch>
          <Match when={isGameAdmin()}>
            <Button ghost disabled class="w-full" justify="start">
              <span class="icon-[fluent--person-settings-20-filled] w-5 h-5 text-error" />
              <span>{t("game.adminMode")}</span>
            </Button>
          </Match>
          <Match when={gameStore.team}>
            <Button ghost class="w-full" justify="start">
              <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-primary" />
              <span class="flex-1 text-start truncate">{gameStore.team?.name}</span>
              <span class="text-success">{gameStore.team?.score} pts</span>
              <span class="text-warning">#{gameStore.rank}</span>
            </Button>
          </Match>
        </Switch>
      </div>
    </>
  );
}
