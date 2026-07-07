// BITs2CTF fork: ISW range-mode admin page — manage hosts, templates, ranges and
// their lifecycle (arm / snapshot / reset). DevOps-gated by the admin layout.
import {
  useArmRangeMutation,
  useCreateHostMutation,
  useCreateRangeMutation,
  useCreateTemplateMutation,
  useDeleteHostMutation,
  useDeleteRangeMutation,
  useIswHosts,
  useProbeHostMutation,
  useResetRangeMutation,
  useSnapshotRangeMutation,
} from "@api/range";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Input from "@widgets/input";
import { createSignal, For, Show } from "solid-js";

export default function () {
  const hosts = useIswHosts();
  const createHost = useCreateHostMutation();
  const deleteHost = useDeleteHostMutation();
  const probeHost = useProbeHostMutation();
  const createTemplate = useCreateTemplateMutation();
  const createRange = useCreateRangeMutation();
  const armRange = useArmRangeMutation();
  const snapshotRange = useSnapshotRangeMutation();
  const resetRange = useResetRangeMutation();
  const deleteRange = useDeleteRangeMutation();

  const [host, setHost] = createSignal({ name: "", address: "", api_port: 8443, os: "windows" });
  const [tpl, setTpl] = createSignal({ game_id: 0, name: "", topology: "" });
  const [range, setRange] = createSignal({ template_id: 0, host_id: 0, group_index: 0, name: "" });
  const [opId, setOpId] = createSignal(0);

  function submitTemplate() {
    let topology: unknown;
    try {
      topology = JSON.parse(tpl().topology || "{}");
    } catch {
      addToast({ level: "error", description: t("range.template.badJson"), duration: 4000 });
      return;
    }
    createTemplate.mutate({
      game_id: tpl().game_id,
      name: tpl().name,
      brief: "",
      topology,
    });
  }

  return (
    <div class="w-full h-full overflow-auto p-3 lg:p-6 flex flex-col space-y-4">
      <Title page={t("range.title")} route="/admin/range" />
      <h1 class="text-xl font-bold">{t("range.title")}</h1>

      {/* Hosts */}
      <Card contentClass="p-3 flex flex-col space-y-3">
        <span class="font-bold">{t("range.host.title")}</span>
        <For each={hosts.data ?? []}>
          {(h) => (
            <div class="flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
              <span class="font-mono flex-1">
                #{h.id} {h.name} — {h.address}:{h.api_port} [{h.os}]
              </span>
              <span class="text-sm opacity-70">
                {h.status}
                <Show when={h.free_mem_mb != null}> · {h.free_mem_mb}MiB</Show>
              </span>
              <Button ghost onClick={() => probeHost.mutate(h.id)}>
                <span class="icon-[fluent--pulse-20-regular] w-5 h-5" />
                <span>{t("range.host.probe")}</span>
              </Button>
              <Button ghost onClick={() => deleteHost.mutate(h.id)}>
                <span class="icon-[fluent--delete-20-regular] w-5 h-5" />
              </Button>
            </div>
          )}
        </For>
        <div class="grid grid-cols-fit-xs gap-2">
          <Input
            title={t("range.host.name")}
            value={host().name}
            onInput={(e) => setHost((h) => ({ ...h, name: e.currentTarget.value }))}
          />
          <Input
            title={t("range.host.address")}
            value={host().address}
            onInput={(e) => setHost((h) => ({ ...h, address: e.currentTarget.value }))}
          />
          <Input
            type="number"
            title={t("range.host.port")}
            value={host().api_port}
            onInput={(e) => setHost((h) => ({ ...h, api_port: Number(e.currentTarget.value) || 8443 }))}
          />
          <Input
            title={t("range.host.os")}
            value={host().os}
            onInput={(e) => setHost((h) => ({ ...h, os: e.currentTarget.value }))}
          />
        </div>
        <Button onClick={() => createHost.mutate(host())}>{t("range.host.add")}</Button>
      </Card>

      {/* Templates */}
      <Card contentClass="p-3 flex flex-col space-y-3">
        <span class="font-bold">{t("range.template.title")}</span>
        <div class="grid grid-cols-fit-xs gap-2">
          <Input
            type="number"
            title={t("range.template.gameId")}
            value={tpl().game_id}
            onInput={(e) => setTpl((s) => ({ ...s, game_id: Number(e.currentTarget.value) || 0 }))}
          />
          <Input
            title={t("range.template.name")}
            value={tpl().name}
            onInput={(e) => setTpl((s) => ({ ...s, name: e.currentTarget.value }))}
          />
        </div>
        <textarea
          class="w-full h-32 font-mono text-sm rounded-md bg-layer-content/5 p-2"
          placeholder={t("range.template.topology")}
          value={tpl().topology}
          onInput={(e) => setTpl((s) => ({ ...s, topology: e.currentTarget.value }))}
        />
        <Button onClick={() => submitTemplate()}>{t("range.template.create")}</Button>
      </Card>

      {/* Ranges */}
      <Card contentClass="p-3 flex flex-col space-y-3">
        <span class="font-bold">{t("range.instance.title")}</span>
        <div class="grid grid-cols-fit-xs gap-2">
          <Input
            type="number"
            title={t("range.instance.templateId")}
            value={range().template_id}
            onInput={(e) => setRange((s) => ({ ...s, template_id: Number(e.currentTarget.value) || 0 }))}
          />
          <Input
            type="number"
            title={t("range.instance.hostId")}
            value={range().host_id}
            onInput={(e) => setRange((s) => ({ ...s, host_id: Number(e.currentTarget.value) || 0 }))}
          />
          <Input
            type="number"
            title={t("range.instance.group")}
            value={range().group_index}
            onInput={(e) => setRange((s) => ({ ...s, group_index: Number(e.currentTarget.value) || 0 }))}
          />
          <Input
            title={t("range.instance.name")}
            value={range().name}
            onInput={(e) => setRange((s) => ({ ...s, name: e.currentTarget.value }))}
          />
        </div>
        <Button onClick={() => createRange.mutate(range())}>{t("range.instance.create")}</Button>
        <div class="flex flex-row flex-wrap gap-2 items-center border-t border-t-layer-content/10 pt-2">
          <Input
            type="number"
            title={t("range.instance.operateId")}
            value={opId()}
            onInput={(e) => setOpId(Number(e.currentTarget.value) || 0)}
          />
          <Button onClick={() => snapshotRange.mutate(opId())}>{t("range.actions.snapshot")}</Button>
          <Button onClick={() => armRange.mutate(opId())}>{t("range.actions.arm")}</Button>
          <Button onClick={() => resetRange.mutate(opId())}>{t("range.actions.reset")}</Button>
          <Button ghost onClick={() => deleteRange.mutate(opId())}>
            {t("general.actions.delete.title")}
          </Button>
        </div>
      </Card>
    </div>
  );
}
