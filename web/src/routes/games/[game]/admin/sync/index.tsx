import { useGame, useGameSyncStatus } from "@api/game";
import {
  useAdvertiseGameSyncMutation,
  useDetachGameSyncMutation,
  useGameSyncReleases,
  usePublishGameSyncMutation,
  useRotateGameSyncTokenMutation,
} from "@api/sync";
import GameSyncReadonlyBanner from "@lib/blocks/game/sync-readonly-banner";
import { HostType } from "@models/game";
import type { RegistryPublicationMetadata, RegistryUpstreamMetadata } from "@models/sync";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import { createMemo, createSignal, For, Show } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });
  const syncStatus = useGameSyncStatus({ game_id: gameId, enabled: () => gameId() > 0 });
  const releases = useGameSyncReleases({ game_id: gameId, enabled: () => gameId() > 0 });
  const [publicationMetadata, setPublicationMetadata] = createSignal<RegistryPublicationMetadata | null>(null);
  const [upstreamMetadata, setUpstreamMetadata] = createSignal<RegistryUpstreamMetadata | null>(null);

  const publishMutation = usePublishGameSyncMutation({
    onSuccess: (metadata) => {
      setPublicationMetadata(metadata);
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
  const detachMutation = useDetachGameSyncMutation({
    onSuccess: () => {
      syncStatus.refetch();
    },
  });
  const advertiseMutation = useAdvertiseGameSyncMutation({
    onSuccess: (metadata) => {
      setUpstreamMetadata(metadata);
      syncStatus.refetch();
    },
  });

  const canPublish = createMemo(
    () => !!game.data && game.data.host_type === HostType.Game && !syncStatus.data?.readonly
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

          <Show when={syncStatus.data?.remote_state}>
            <Card contentClass="p-4 flex flex-col space-y-3">
              <div class="flex flex-row items-center space-x-2 font-bold">
                <span class="shrink-0 icon-[fluent--shield-lock-20-regular] w-5 h-5" />
                <span>{t("game.sync.remote.title")}</span>
              </div>
              <div class="flex flex-col gap-1 text-sm">
                <span>
                  {t("game.sync.remote.state", {
                    state:
                      syncStatus.data?.remote_state === "mirror_locked"
                        ? t("game.sync.remote.stateMirrorLocked")
                        : t("game.sync.remote.stateDetached"),
                  })}
                </span>
                <Show when={syncStatus.data?.remote_release_id}>
                  <span>
                    {t("game.sync.remote.release", {
                      value: syncStatus.data?.remote_release_id || "-",
                    })}
                  </span>
                </Show>
                <Show when={syncStatus.data?.remote_first_party_base_url}>
                  <span>
                    {t("game.sync.remote.source", {
                      value: syncStatus.data?.remote_first_party_base_url || "-",
                    })}
                  </span>
                </Show>
              </div>
              <Show when={syncStatus.data?.remote_state === "mirror_locked"}>
                <p class="opacity-80 text-sm">{t("game.sync.remote.detachDescription")}</p>
                <div class="flex flex-row justify-end">
                  <Button
                    level="warning"
                    onClick={() => detachMutation.mutate({ game_id: gameId() })}
                    loading={detachMutation.isPending}
                    disabled={detachMutation.isPending}
                  >
                    {t("game.sync.actions.detach.title")}
                  </Button>
                </div>
              </Show>
            </Card>
          </Show>

          <Show when={syncStatus.data?.remote_state === "mirror_locked"}>
            <Card contentClass="p-4 flex flex-col space-y-3">
              <div class="flex flex-row items-center space-x-2 font-bold">
                <span class="shrink-0 icon-[fluent--share-screen-person-20-regular] w-5 h-5" />
                <span>{t("game.sync.advertise.title")}</span>
              </div>
              <p class="opacity-80 text-sm">{t("game.sync.advertise.description")}</p>
              <div class="flex flex-row justify-end">
                <Button
                  level="primary"
                  onClick={() => advertiseMutation.mutate({ game_id: gameId() })}
                  loading={advertiseMutation.isPending}
                  disabled={advertiseMutation.isPending}
                >
                  {t("game.sync.actions.advertise.title")}
                </Button>
              </div>
            </Card>
          </Show>

          <Card contentClass="p-4 flex flex-col space-y-3">
            <div class="flex flex-row items-center space-x-2 font-bold">
              <span class="shrink-0 icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5" />
              <span>{t("game.sync.publish.title")}</span>
            </div>
            <p class="opacity-80 text-sm">{t("game.sync.publish.description")}</p>
            <div class="flex flex-row justify-end">
              <Button
                level="primary"
                onClick={() => publishMutation.mutate({ game_id: gameId() })}
                loading={publishMutation.isPending}
                disabled={!canPublish() || publishMutation.isPending}
              >
                {t("game.sync.actions.publish.title")}
              </Button>
            </div>
          </Card>

          <Show when={publicationMetadata()}>
            {(metadata) => (
              <Card contentClass="p-4 flex flex-col space-y-3">
                <div class="flex flex-row items-center space-x-2 font-bold">
                  <span class="shrink-0 icon-[fluent--document-text-20-regular] w-5 h-5" />
                  <span>{t("game.sync.manualPr.title")}</span>
                </div>
                <p class="opacity-80 text-sm">{t("game.sync.manualPr.description")}</p>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{metadata().release_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {metadata().release_file_content}
                  </pre>
                </div>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{metadata().upstream_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {metadata().upstream_file_content}
                  </pre>
                </div>
              </Card>
            )}
          </Show>

          <Show when={upstreamMetadata()}>
            {(metadata) => (
              <Card contentClass="p-4 flex flex-col space-y-3">
                <div class="flex flex-row items-center space-x-2 font-bold">
                  <span class="shrink-0 icon-[fluent--document-text-20-regular] w-5 h-5" />
                  <span>{t("game.sync.manualUpstream.title")}</span>
                </div>
                <p class="opacity-80 text-sm">{t("game.sync.manualUpstream.description")}</p>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{metadata().upstream_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {metadata().upstream_file_content}
                  </pre>
                </div>
              </Card>
            )}
          </Show>

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
