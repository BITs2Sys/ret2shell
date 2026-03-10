import {
  type SyncRegistrySourcePayload,
  useCreateSyncSourceMutation,
  useDeleteSyncSourceMutation,
  useDiscoverDirectSyncMutation,
  useFetchSyncSourceMutation,
  useImportDirectSyncMutation,
  useSyncSources,
  useUpdateSyncSourceMutation,
} from "@api/sync";
import type { DirectDiscoverResponse, SyncRegistrySource } from "@models/sync";
import { useNavigate } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Input from "@widgets/input";
import { createMemo, createSignal, For, Show } from "solid-js";

const EMPTY_FORM: SyncRegistrySourcePayload = {
  name: "",
  git_url: "",
  branch: "main",
  enabled: true,
  priority: 0,
  publish_enabled: false,
  private_source: false,
};

export default function () {
  const navigate = useNavigate();
  const sources = useSyncSources();
  const [editingId, setEditingId] = createSignal<number | null>(null);
  const [form, setForm] = createSignal<SyncRegistrySourcePayload>({ ...EMPTY_FORM });
  const [discoverBaseUrl, setDiscoverBaseUrl] = createSignal("");
  const [discoverSyncToken, setDiscoverSyncToken] = createSignal("");
  const [discoverGameKey, setDiscoverGameKey] = createSignal("");
  const [discoverReleaseId, setDiscoverReleaseId] = createSignal("");
  const [discoverResult, setDiscoverResult] = createSignal<DirectDiscoverResponse | null>(null);

  const createMutation = useCreateSyncSourceMutation({
    onSuccess: () => {
      sources.refetch();
      resetForm();
    },
  });
  const updateMutation = useUpdateSyncSourceMutation({
    onSuccess: () => {
      sources.refetch();
      resetForm();
    },
  });
  const deleteMutation = useDeleteSyncSourceMutation({
    onSuccess: () => {
      sources.refetch();
      if (editingId() != null && !sources.data?.find((source) => source.id === editingId())) {
        resetForm();
      }
    },
  });
  const fetchMutation = useFetchSyncSourceMutation({
    onSuccess: () => {
      sources.refetch();
    },
  });
  const discoverMutation = useDiscoverDirectSyncMutation({
    onSuccess: (data) => {
      setDiscoverResult(data);
    },
  });
  const importMutation = useImportDirectSyncMutation({
    onSuccess: (data) => {
      navigate(`/games/${data.game_id}/admin/sync`);
    },
  });

  const isEditing = createMemo(() => editingId() != null);

  function updateForm<K extends keyof SyncRegistrySourcePayload>(key: K, value: SyncRegistrySourcePayload[K]) {
    setForm((current) => ({
      ...current,
      [key]: value,
    }));
  }

  function resetForm() {
    setEditingId(null);
    setForm({ ...EMPTY_FORM });
  }

  function beginEdit(source: SyncRegistrySource) {
    setEditingId(source.id);
    setForm({
      name: source.name,
      git_url: source.git_url,
      branch: source.branch,
      enabled: source.enabled,
      priority: source.priority,
      publish_enabled: source.publish_enabled,
      private_source: source.private_source,
    });
  }

  function onSubmit() {
    if (!form().name.trim() || !form().git_url.trim() || !form().branch.trim()) {
      return;
    }
    if (isEditing()) {
      updateMutation.mutate({ id: editingId()!, source: form() });
    } else {
      createMutation.mutate(form());
    }
  }

  function onDiscover() {
    discoverMutation.mutate({
      base_url: discoverBaseUrl(),
      sync_token: discoverSyncToken() || null,
      game_key: discoverGameKey() || null,
      release_id: discoverReleaseId() || null,
    });
  }

  function onImport() {
    if (!discoverGameKey().trim() || !discoverReleaseId().trim()) {
      return;
    }
    importMutation.mutate({
      base_url: discoverBaseUrl(),
      sync_token: discoverSyncToken() || null,
      game_key: discoverGameKey().trim(),
      release_id: discoverReleaseId().trim(),
    });
  }

  return (
    <>
      <Title page={t("platform.sync.title")} route="/admin/sync" />
      <div class="flex-1 flex flex-col p-3 lg:p-6 space-y-3">
        <Card contentClass="p-4 flex flex-col space-y-3">
          <div class="flex flex-row items-center space-x-2 font-bold">
            <span class="shrink-0 icon-[fluent--database-plug-connected-20-regular] w-5 h-5" />
            <span>{t("platform.sync.sources.title")}</span>
          </div>
          <p class="opacity-80">{t("platform.sync.sources.description")}</p>
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-2">
            <Input
              value={form().name}
              onInput={(event) => updateForm("name", event.currentTarget.value)}
              title={t("platform.sync.sources.form.name")}
              placeholder={t("platform.sync.sources.form.name")}
              icon={<span class="shrink-0 icon-[fluent--tag-20-regular] w-5 h-5" />}
            />
            <Input
              value={form().branch}
              onInput={(event) => updateForm("branch", event.currentTarget.value)}
              title={t("platform.sync.sources.form.branch")}
              placeholder="main"
              icon={<span class="shrink-0 icon-[fluent--branch-fork-20-regular] w-5 h-5" />}
            />
          </div>
          <Input
            value={form().git_url}
            onInput={(event) => updateForm("git_url", event.currentTarget.value)}
            title={t("platform.sync.sources.form.gitUrl")}
            placeholder="https://github.com/ret2shell/game-registry"
            icon={<span class="shrink-0 icon-[fluent--link-20-regular] w-5 h-5" />}
          />
          <div class="grid grid-cols-1 lg:grid-cols-4 gap-2">
            <Input
              type="number"
              value={form().priority}
              onInput={(event) => updateForm("priority", Number.parseInt(event.currentTarget.value || "0", 10) || 0)}
              title={t("platform.sync.sources.form.priority")}
              placeholder="0"
              icon={<span class="shrink-0 icon-[fluent--arrow-sort-20-regular] w-5 h-5" />}
            />
            <Checkbox checked={form().enabled} onChange={() => updateForm("enabled", !form().enabled)}>
              <span class="flex-1 text-start">{t("platform.sync.sources.form.enabled")}</span>
            </Checkbox>
            <Checkbox
              checked={form().publish_enabled}
              onChange={() => updateForm("publish_enabled", !form().publish_enabled)}
            >
              <span class="flex-1 text-start">{t("platform.sync.sources.form.publishEnabled")}</span>
            </Checkbox>
            <Checkbox
              checked={form().private_source}
              onChange={() => updateForm("private_source", !form().private_source)}
            >
              <span class="flex-1 text-start">{t("platform.sync.sources.form.privateSource")}</span>
            </Checkbox>
          </div>
          <div class="flex flex-row justify-end space-x-2">
            <Show when={isEditing()}>
              <Button ghost onClick={resetForm}>
                {t("general.actions.cancel.title")}
              </Button>
            </Show>
            <Button
              level="primary"
              onClick={onSubmit}
              loading={createMutation.isPending || updateMutation.isPending}
              disabled={createMutation.isPending || updateMutation.isPending}
            >
              {isEditing() ? t("general.actions.save.title") : t("general.actions.create.title")}
            </Button>
          </div>
        </Card>

        <Card contentClass="p-4 flex flex-col space-y-3">
          <div class="flex flex-row items-center space-x-2 font-bold">
            <span class="shrink-0 icon-[fluent--globe-arrow-up-20-regular] w-5 h-5" />
            <span>{t("platform.sync.direct.title")}</span>
          </div>
          <p class="opacity-80">{t("platform.sync.direct.description")}</p>
          <Input
            value={discoverBaseUrl()}
            onInput={(event) => setDiscoverBaseUrl(event.currentTarget.value)}
            title={t("platform.sync.direct.form.baseUrl")}
            placeholder="https://ctf.example.com"
            icon={<span class="shrink-0 icon-[fluent--link-20-regular] w-5 h-5" />}
          />
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-2">
            <Input
              value={discoverSyncToken()}
              onInput={(event) => setDiscoverSyncToken(event.currentTarget.value)}
              title={t("platform.sync.direct.form.syncToken")}
              placeholder="r2s_sync_xxxxx"
              icon={<span class="shrink-0 icon-[fluent--key-20-regular] w-5 h-5" />}
            />
            <Input
              value={discoverGameKey()}
              onInput={(event) => setDiscoverGameKey(event.currentTarget.value)}
              title={t("platform.sync.direct.form.gameKey")}
              placeholder="example_game"
              icon={<span class="shrink-0 icon-[fluent--cube-20-regular] w-5 h-5" />}
            />
            <Input
              value={discoverReleaseId()}
              onInput={(event) => setDiscoverReleaseId(event.currentTarget.value)}
              title={t("platform.sync.direct.form.releaseId")}
              placeholder="commit id"
              icon={<span class="shrink-0 icon-[fluent--history-20-regular] w-5 h-5" />}
            />
          </div>
          <div class="flex flex-row justify-end">
            <div class="flex flex-row gap-2">
              <Button
                level="primary"
                onClick={onDiscover}
                loading={discoverMutation.isPending}
                disabled={discoverMutation.isPending}
              >
                {t("platform.sync.direct.action")}
              </Button>
              <Button
                onClick={onImport}
                loading={importMutation.isPending}
                disabled={importMutation.isPending || !discoverGameKey().trim() || !discoverReleaseId().trim()}
              >
                {t("platform.sync.direct.import.title")}
              </Button>
            </div>
          </div>
          <Show when={discoverResult()}>
            {(result) => (
              <div class="flex flex-col space-y-3">
                <div class="text-sm opacity-80">
                  {t("platform.sync.direct.info", {
                    baseUrl: result().info.base_url,
                    version: result().info.protocol_version,
                  })}
                </div>
                <Show when={result().games?.length}>
                  <div class="flex flex-col space-y-2">
                    <div class="font-bold">{t("platform.sync.direct.games")}</div>
                    <For each={result().games || []}>
                      {(game) => (
                        <button
                          type="button"
                          class="border border-layer-content/10 rounded-lg p-3 flex flex-row justify-between gap-2 text-start hover:border-primary/40 transition-colors"
                          onClick={() => {
                            setDiscoverGameKey(game.game_key);
                            setDiscoverReleaseId("");
                          }}
                        >
                          <span class="font-mono break-all">{game.game_key}</span>
                          <span class="opacity-70 text-sm">
                            {t("platform.sync.direct.releaseCount", { count: game.release_count })}
                          </span>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
                <Show when={result().releases?.length}>
                  <div class="flex flex-col space-y-2">
                    <div class="font-bold">{t("platform.sync.direct.releases")}</div>
                    <For each={result().releases || []}>
                      {(release) => (
                        <button
                          type="button"
                          class="border border-layer-content/10 rounded-lg p-3 flex flex-col gap-1 text-start hover:border-primary/40 transition-colors"
                          onClick={() => {
                            setDiscoverGameKey(release.game_key);
                            setDiscoverReleaseId(release.release_id);
                          }}
                        >
                          <span class="font-mono break-all">{release.release_id}</span>
                          <span class="text-sm opacity-70">{release.first_party_base_url}</span>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
                <Show when={result().release}>
                  {(release) => (
                    <div class="border border-layer-content/10 rounded-lg p-3 flex flex-col gap-1">
                      <span class="font-mono break-all">{release().release_id}</span>
                      <span class="text-sm opacity-70">{release().snapshot_commit}</span>
                      <span class="text-sm opacity-70">{release().first_party_base_url}</span>
                    </div>
                  )}
                </Show>
              </div>
            )}
          </Show>
        </Card>

        <For
          each={sources.data || []}
          fallback={
            <Card contentClass="p-4 opacity-70 flex flex-row items-center space-x-2">
              <span class="shrink-0 icon-[fluent--box-20-regular] w-5 h-5" />
              <span>{t("platform.sync.sources.empty")}</span>
            </Card>
          }
        >
          {(source) => (
            <Card contentClass="p-4 flex flex-col space-y-3">
              <div class="flex flex-row items-start gap-2">
                <div class="flex-1 min-w-0 flex flex-col">
                  <div class="font-bold truncate">{source.name}</div>
                  <div class="opacity-70 truncate">{source.git_url}</div>
                </div>
                <div class="flex flex-row gap-2">
                  <Button size="sm" ghost onClick={() => beginEdit(source)}>
                    {t("general.actions.edit.title")}
                  </Button>
                  <Button
                    size="sm"
                    onClick={() => fetchMutation.mutate({ id: source.id })}
                    loading={fetchMutation.isPending}
                    disabled={fetchMutation.isPending}
                  >
                    {t("platform.sync.sources.actions.fetch.title")}
                  </Button>
                  <Button
                    size="sm"
                    level="error"
                    ghost
                    onClick={() => deleteMutation.mutate({ id: source.id })}
                    loading={deleteMutation.isPending}
                    disabled={deleteMutation.isPending}
                  >
                    {t("general.actions.delete.title")}
                  </Button>
                </div>
              </div>
              <div class="grid grid-cols-1 lg:grid-cols-4 gap-2 text-sm opacity-80">
                <span>
                  {t("platform.sync.sources.form.branch")}: {source.branch}
                </span>
                <span>
                  {t("platform.sync.sources.form.priority")}: {source.priority}
                </span>
                <span>
                  {t("platform.sync.sources.badges.enabled", {
                    enabled: source.enabled
                      ? t("platform.sync.sources.state.enabled")
                      : t("platform.sync.sources.state.disabled"),
                  })}
                </span>
                <span>
                  {t("platform.sync.sources.badges.publishEnabled", {
                    enabled: source.publish_enabled
                      ? t("platform.sync.sources.state.enabled")
                      : t("platform.sync.sources.state.disabled"),
                  })}
                </span>
              </div>
              <Show when={source.last_fetched_at}>
                <span class="text-sm opacity-70">
                  {t("platform.sync.sources.lastFetched", {
                    value: source.last_fetched_at?.toFormat("yyyy-MM-dd HH:mm:ss") || "-",
                  })}
                </span>
              </Show>
              <Show when={source.last_error}>
                <div class="text-sm text-error whitespace-pre-wrap break-all">{source.last_error}</div>
              </Show>
            </Card>
          )}
        </For>
      </div>
    </>
  );
}
