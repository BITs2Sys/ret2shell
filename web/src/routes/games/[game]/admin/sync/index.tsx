import { useGame, useGameSyncStatus } from "@api/game";
import {
  useGameSyncReleases,
  useGameSyncSources,
  usePublishGameSyncMutation,
  useRotateGameSyncTokenMutation,
} from "@api/sync";
import GameSyncReadonlyBanner from "@lib/blocks/game/sync-readonly-banner";
import { HostType } from "@models/game";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Select from "@widgets/select";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });
  const syncStatus = useGameSyncStatus({ game_id: gameId, enabled: () => gameId() > 0 });
  const releases = useGameSyncReleases({ game_id: gameId, enabled: () => gameId() > 0 });
  const sources = useGameSyncSources({ game_id: gameId, enabled: () => gameId() > 0 });
  const [selectedSourceId, setSelectedSourceId] = createSignal<string>("");

  createEffect(() => {
    if (selectedSourceId() || !sources.data?.length) {
      return;
    }
    const defaultSource = sources.data.find((source) => source.publish_enabled && source.enabled) ?? sources.data[0];
    if (defaultSource) {
      setSelectedSourceId(defaultSource.id.toString());
    }
  });

  const publishMutation = usePublishGameSyncMutation({
    onSuccess: () => {
      releases.refetch();
      syncStatus.refetch();
    },
  });
  const rotateTokenMutation = useRotateGameSyncTokenMutation({
    onSuccess: () => {
      syncStatus.refetch();
      game.refetch();
    },
  });

  const publishableSources = createMemo(
    () => sources.data?.filter((source) => source.publish_enabled && source.enabled) || []
  );
  const canPublish = createMemo(
    () =>
      !!game.data &&
      game.data.host_type === HostType.Game &&
      !!selectedSourceId() &&
      publishableSources().length > 0 &&
      !syncStatus.data?.readonly
  );

  return (
    <>
      <Title page={t("game.sync.title")} route={`/games/${gameId()}/admin/sync`} />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center space-y-3">
        <div class="w-full max-w-5xl space-y-3">
          <GameSyncReadonlyBanner gameId={gameId()} />

          <Card contentClass="p-4 flex flex-col space-y-3">
            <div class="flex flex-row items-center space-x-2 font-bold">
              <span class="shrink-0 icon-[fluent--key-20-regular] w-5 h-5" />
              <span>{t("game.sync.identity.title")}</span>
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-2 text-sm">
              <div>
                <span class="opacity-70">{t("game.sync.identity.syncKey")}: </span>
                <span class="font-mono break-all">{syncStatus.data?.sync_key || "-"}</span>
              </div>
              <div>
                <span class="opacity-70">{t("game.sync.identity.syncToken")}: </span>
                <span class="font-mono break-all">{syncStatus.data?.sync_token || "-"}</span>
              </div>
            </div>
            <div class="flex flex-row justify-end">
              <Button
                onClick={() => rotateTokenMutation.mutate({ game_id: gameId() })}
                loading={rotateTokenMutation.isPending}
                disabled={rotateTokenMutation.isPending || syncStatus.data?.readonly}
              >
                {t("game.sync.actions.rotateToken.title")}
              </Button>
            </div>
          </Card>

          <Card contentClass="p-4 flex flex-col space-y-3">
            <div class="flex flex-row items-center space-x-2 font-bold">
              <span class="shrink-0 icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5" />
              <span>{t("game.sync.publish.title")}</span>
            </div>
            <p class="opacity-80 text-sm">{t("game.sync.publish.description")}</p>
            <Select
              label={t("game.sync.publish.source")}
              placeholder={t("game.sync.publish.sourcePlaceholder")}
              items={publishableSources().map((source) => ({
                label: `${source.name} (${source.branch})`,
                value: source.id.toString(),
              }))}
              value={selectedSourceId() ? [selectedSourceId()] : []}
              onValueChange={(details) => {
                setSelectedSourceId(details.value[0] || "");
              }}
              disabled={!publishableSources().length || syncStatus.data?.readonly}
            />
            <div class="flex flex-row justify-end">
              <Button
                level="primary"
                onClick={() =>
                  publishMutation.mutate({
                    game_id: gameId(),
                    registry_source_id: Number.parseInt(selectedSourceId() || "0", 10),
                  })
                }
                loading={publishMutation.isPending}
                disabled={!canPublish() || publishMutation.isPending}
              >
                {t("game.sync.actions.publish.title")}
              </Button>
            </div>
          </Card>

          <Card contentClass="p-4 flex flex-col space-y-3">
            <div class="flex flex-row items-center space-x-2 font-bold">
              <span class="shrink-0 icon-[fluent--history-20-regular] w-5 h-5" />
              <span>{t("game.sync.releases.title")}</span>
            </div>
            <For each={releases.data || []} fallback={<span class="opacity-70">{t("game.sync.releases.empty")}</span>}>
              {(release) => (
                <div class="border border-layer-content/10 rounded-lg p-3 flex flex-col space-y-1">
                  <div class="font-mono break-all">{release.release_id}</div>
                  <div class="text-sm opacity-80">
                    {t("game.sync.releases.snapshot", { value: release.snapshot_commit })}
                  </div>
                  <div class="text-sm opacity-80">
                    {t("game.sync.releases.publishedAt", {
                      value: release.published_at.toFormat("yyyy-MM-dd HH:mm:ss"),
                    })}
                  </div>
                </div>
              )}
            </For>
          </Card>

          <Show when={game.data?.host_type !== HostType.Game}>
            <Card level="warning" contentClass="p-4">
              <span>{t("game.sync.trainingHint")}</span>
            </Card>
          </Show>
        </div>
      </div>
    </>
  );
}
