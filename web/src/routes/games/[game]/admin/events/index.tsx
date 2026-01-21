import { useGame, useGameDevices, useRegenerateGameTokenMutation } from "@api/game";
import { createBreakpoints } from "@solid-primitives/media";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { breakpoints, t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Clipboard from "@widgets/clipboard";
import Popover from "@widgets/popover";
import { createMemo, For, Match, Switch } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });
  const devices = useGameDevices({ game_id: gameId, enabled: () => gameId() > 0 });
  const tokenMutation = useRegenerateGameTokenMutation({
    onSuccess: () => {
      game.refetch();
      devices.refetch();
    },
  });

  async function handleRefreshToken() {
    if (gameId() <= 0) return;
    tokenMutation.mutate({ id: gameId() });
  }

  const matches = createBreakpoints(breakpoints);
  return (
    <>
      <Title page={t("game.events.title")} route={`/games/${gameId()}/admin/events`} />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6 relative">
        <div class="flex-1 flex flex-col w-full max-w-5xl">
          <h2 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--cloud-flow-20-regular] w-5 h-5" />
            <span>{t("game.events.title")} API</span>
          </h2>
          <Card level="info" class="mt-2" contentClass="p-2 flex flex-row space-x-2 items-center">
            <span class="shrink-0 icon-[fluent--info-20-regular] w-5 h-5" />
            <p class="opacity-60 inline">
              <span>{t("game.events.howto")}</span>
              <span>&nbsp;</span>
              <a
                href="https://docs.ret.sh.cn/"
                class="inline-flex flex-row space-x-2 items-center hover:underline"
                target="_blank"
                rel="noreferrer"
              >
                <span>{t("docs.title")}</span>
                <span class="shrink-0 icon-[fluent--open-16-regular] w-4 h-4 text-primary" />
              </a>
              <span>.&nbsp;</span>
              <span>{t("game.events.tokenWarning")}</span>
            </p>
          </Card>
          <div class="flex flex-row space-x-2 mt-2">
            <Clipboard
              class="flex-1"
              value={`${window.origin.replace("http", "ws")}/api/event/connect?game_id=${gameId()}&token=${game.data?.token || undefined}`}
            />
            <Popover
              square
              btnContent={<span class="shrink-0 icon-[fluent--arrow-sync-20-regular] text-error w-5 h-5" />}
            >
              <Card contentClass="p-2 flex flex-col space-y-2 max-w-96">
                <span class="inline-block space-x-2">
                  <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                  <span>{t("game.events.regenerateTokenWarning")}</span>
                </span>
                <Button level="primary" size="sm" class="self-end" onClick={handleRefreshToken}>
                  {t("general.actions.yes.title")}
                </Button>
              </Card>
            </Popover>
          </div>
          <div class="h-4" />
          <h2 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--developer-board-lightning-20-regular] w-5 h-5" />
            <span>{t("game.events.linkedDevices")}</span>
          </h2>
          <For
            each={devices.data || []}
            fallback={
              <div class="flex-1 flex flex-col items-center justify-center space-y-8 opacity-60">
                <span class="shrink-0 icon-[fluent--desktop-20-regular] w-24 h-24" />
                <span>{t("game.events.emptyDevices")}</span>
              </div>
            }
          >
            {(device) => (
              <>
                <div class="h-12 border-b border-b-layer-content/10 flex flex-row items-center space-x-4 px-2">
                  <span class="w-2 h-2 mx-2 rounded-full bg-success" />
                  <span class="font-bold">{device.address}</span>
                  <span class="flex-1 opacity-60 truncate" title={device.client}>
                    {device.client}
                  </span>
                  <span class="shrink-0">
                    <Switch fallback={device.connected_at.toFormat("MM-dd HH:mm")}>
                      <Match when={matches.sm}>{device.connected_at.toFormat("yyyy-MM-dd HH:mm:ss")}</Match>
                      <Match when={matches.xs}>{device.connected_at.toFormat("MM-dd HH:mm:ss")}</Match>
                    </Switch>
                  </span>
                </div>
              </>
            )}
          </For>
        </div>
        {/* <NarrowTips breakpoint="md" /> */}
      </div>
    </>
  );
}
