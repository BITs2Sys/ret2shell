import { useGame, useGameSyncStatus } from "@api/game";
import {
  useAdvertiseGameSyncMutation,
  useDetachGameSyncMutation,
  useGameSyncReleases,
  useGameSyncSources,
  usePublishGameSyncMutation,
  useRevokeGameSyncMutation,
  useRotateGameSyncTokenMutation,
} from "@api/sync";
import GameSyncReadonlyBanner from "@lib/blocks/game/sync-readonly-banner";
import { HostType } from "@models/game";
import type { ManualRegistryPublication, ManualRegistryUpstreamPublication } from "@models/sync";
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
  const [selectedAdvertiseSourceId, setSelectedAdvertiseSourceId] = createSignal<string>("");
  const [selectedRevokeSourceId, setSelectedRevokeSourceId] = createSignal<string>("");
  const [manualPublication, setManualPublication] = createSignal<ManualRegistryPublication | null>(null);
  const [manualUpstreamPublication, setManualUpstreamPublication] =
    createSignal<ManualRegistryUpstreamPublication | null>(null);
  const [manualRevocationPublication, setManualRevocationPublication] =
    createSignal<ManualRegistryUpstreamPublication | null>(null);

  createEffect(() => {
    if (selectedSourceId() || !sources.data?.length) {
      return;
    }
    const defaultSource = sources.data.find((source) => source.publish_enabled && source.enabled) ?? sources.data[0];
    if (defaultSource) {
      setSelectedSourceId(defaultSource.id.toString());
    }
  });

  createEffect(() => {
    if (selectedAdvertiseSourceId() || !sources.data?.length) {
      return;
    }
    const defaultSource = sources.data.find((source) => source.publish_enabled && source.enabled) ?? sources.data[0];
    if (defaultSource) {
      setSelectedAdvertiseSourceId(defaultSource.id.toString());
    }
  });

  createEffect(() => {
    if (selectedRevokeSourceId() || !sources.data?.length) {
      return;
    }
    const defaultSource = sources.data.find((source) => source.publish_enabled && source.enabled) ?? sources.data[0];
    if (defaultSource) {
      setSelectedRevokeSourceId(defaultSource.id.toString());
    }
  });

  const publishMutation = usePublishGameSyncMutation({
    onSuccess: (publication) => {
      setManualPublication(publication);
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
    onSuccess: (publication) => {
      setManualUpstreamPublication(publication);
      syncStatus.refetch();
    },
  });
  const revokeMutation = useRevokeGameSyncMutation({
    onSuccess: (publication) => {
      setManualRevocationPublication(publication);
      syncStatus.refetch();
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
              <Select
                label={t("game.sync.advertise.source")}
                placeholder={t("game.sync.advertise.sourcePlaceholder")}
                items={publishableSources().map((source) => ({
                  label: `${source.name} (${source.branch})`,
                  value: source.id.toString(),
                }))}
                value={selectedAdvertiseSourceId() ? [selectedAdvertiseSourceId()] : []}
                onValueChange={(details) => {
                  setSelectedAdvertiseSourceId(details.value[0] || "");
                }}
                disabled={!publishableSources().length}
              />
              <div class="flex flex-row justify-end">
                <Button
                  level="primary"
                  onClick={() =>
                    advertiseMutation.mutate({
                      game_id: gameId(),
                      registry_source_id: Number.parseInt(selectedAdvertiseSourceId() || "0", 10),
                    })
                  }
                  loading={advertiseMutation.isPending}
                  disabled={!selectedAdvertiseSourceId() || advertiseMutation.isPending}
                >
                  {t("game.sync.actions.advertise.title")}
                </Button>
              </div>
            </Card>
          </Show>

          <Show when={syncStatus.data?.remote_state === "detached"}>
            <Card contentClass="p-4 flex flex-col space-y-3">
              <div class="flex flex-row items-center space-x-2 font-bold">
                <span class="shrink-0 icon-[fluent--arrow-undo-20-regular] w-5 h-5" />
                <span>{t("game.sync.revoke.title")}</span>
              </div>
              <p class="opacity-80 text-sm">{t("game.sync.revoke.description")}</p>
              <Select
                label={t("game.sync.revoke.source")}
                placeholder={t("game.sync.revoke.sourcePlaceholder")}
                items={publishableSources().map((source) => ({
                  label: `${source.name} (${source.branch})`,
                  value: source.id.toString(),
                }))}
                value={selectedRevokeSourceId() ? [selectedRevokeSourceId()] : []}
                onValueChange={(details) => {
                  setSelectedRevokeSourceId(details.value[0] || "");
                }}
                disabled={!publishableSources().length}
              />
              <div class="flex flex-row justify-end">
                <Button
                  level="warning"
                  onClick={() =>
                    revokeMutation.mutate({
                      game_id: gameId(),
                      registry_source_id: Number.parseInt(selectedRevokeSourceId() || "0", 10),
                    })
                  }
                  loading={revokeMutation.isPending}
                  disabled={!selectedRevokeSourceId() || revokeMutation.isPending}
                >
                  {t("game.sync.actions.revoke.title")}
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

          <Show when={manualPublication()}>
            {(publication) => (
              <Card contentClass="p-4 flex flex-col space-y-3">
                <div class="flex flex-row items-center space-x-2 font-bold">
                  <span class="shrink-0 icon-[fluent--document-text-20-regular] w-5 h-5" />
                  <span>{t("game.sync.manualPr.title")}</span>
                </div>
                <p class="opacity-80 text-sm">{t("game.sync.manualPr.description")}</p>
                <div class="text-sm flex flex-col gap-1">
                  <span>
                    {t("game.sync.manualPr.target", {
                      repo: publication().registry_git_url,
                      branch: publication().registry_branch,
                    })}
                  </span>
                  <span>
                    {t("game.sync.manualPr.prTitle", {
                      title: publication().suggested_pr_title,
                    })}
                  </span>
                </div>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{publication().release_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {publication().release_file_content}
                  </pre>
                </div>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{publication().upstream_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {publication().upstream_file_content}
                  </pre>
                </div>
              </Card>
            )}
          </Show>

          <Show when={manualUpstreamPublication()}>
            {(publication) => (
              <Card contentClass="p-4 flex flex-col space-y-3">
                <div class="flex flex-row items-center space-x-2 font-bold">
                  <span class="shrink-0 icon-[fluent--document-text-20-regular] w-5 h-5" />
                  <span>{t("game.sync.manualUpstream.title")}</span>
                </div>
                <p class="opacity-80 text-sm">{t("game.sync.manualUpstream.description")}</p>
                <div class="text-sm flex flex-col gap-1">
                  <span>
                    {t("game.sync.manualUpstream.target", {
                      repo: publication().registry_git_url,
                      branch: publication().registry_branch,
                    })}
                  </span>
                  <span>
                    {t("game.sync.manualUpstream.prTitle", {
                      title: publication().suggested_pr_title,
                    })}
                  </span>
                </div>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{publication().upstream_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {publication().upstream_file_content}
                  </pre>
                </div>
              </Card>
            )}
          </Show>

          <Show when={manualRevocationPublication()}>
            {(publication) => (
              <Card contentClass="p-4 flex flex-col space-y-3">
                <div class="flex flex-row items-center space-x-2 font-bold">
                  <span class="shrink-0 icon-[fluent--document-text-20-regular] w-5 h-5" />
                  <span>{t("game.sync.manualRevoke.title")}</span>
                </div>
                <p class="opacity-80 text-sm">{t("game.sync.manualRevoke.description")}</p>
                <div class="text-sm flex flex-col gap-1">
                  <span>
                    {t("game.sync.manualRevoke.target", {
                      repo: publication().registry_git_url,
                      branch: publication().registry_branch,
                    })}
                  </span>
                  <span>
                    {t("game.sync.manualRevoke.prTitle", {
                      title: publication().suggested_pr_title,
                    })}
                  </span>
                </div>
                <div class="flex flex-col gap-2">
                  <span class="font-bold text-sm">{publication().upstream_file_path}</span>
                  <pre class="whitespace-pre-wrap break-all rounded-lg border border-layer-content/10 p-3 text-xs overflow-x-auto">
                    {publication().upstream_file_content}
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
