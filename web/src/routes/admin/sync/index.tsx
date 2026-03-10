import {
  type SyncRegistrySourcePayload,
  useCreateSyncSourceMutation,
  useDeleteSyncSourceMutation,
  useFetchSyncSourceMutation,
  useSyncSources,
  useUpdateSyncSourceMutation,
} from "@api/sync";
import type { SyncRegistrySource } from "@models/sync";
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
  const sources = useSyncSources();
  const [editingId, setEditingId] = createSignal<number | null>(null);
  const [form, setForm] = createSignal<SyncRegistrySourcePayload>({ ...EMPTY_FORM });

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
