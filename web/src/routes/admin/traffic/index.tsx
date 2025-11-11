import { handleHttpError } from "@api";
import {
  deleteDefaultNodeSelector,
  deleteGlobalTrafficScript,
  updateDefaultNodeSelector,
  updateGlobalTrafficScript,
} from "@api/cluster";
import { getPlatformConfig } from "@api/platform";
import type { Config } from "@models/config";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Divider from "@widgets/divider";
import { type DiagnosticMarker, EditorBare } from "@widgets/editor";
import Input from "@widgets/input";
import Popover from "@widgets/popover";
import Select from "@widgets/select";
import { createEffect, createSignal, onMount, Show } from "solid-js";
import multiNodeDirect from "./scripts/multi_node_direct.rx";
import singleNodeDirect from "./scripts/single_node_direct.rx";

type PresetTraffic = "single-node-direct" | "multi-node-direct";

const trafficMap = {
  "single-node-direct": singleNodeDirect,
  "multi-node-direct": multiNodeDirect,
};

export default function Traffic() {
  const [config, setConfig] = createSignal(null as null | Config);
  onMount(async () => {
    try {
      const resp = await getPlatformConfig();
      setConfig(resp);
    } catch (err) {
      handleHttpError(err as Error, t("platform.errors.fetchConfig.title")!);
    }
  });
  const [preset, setPreset] = createSignal(null as PresetTraffic | null);

  const [script, setScript] = createSignal("");
  const [lint, setLint] = createSignal(null as DiagnosticMarker[] | null);
  const [nodeSelector, setNodeSelector] = createSignal("");
  const [saving, setSaving] = createSignal(false);
  createEffect(() => {
    if (preset()) {
      setScript(trafficMap[preset()!]);
    }
  });

  createEffect(() => {
    if (config()?.cluster) {
      setScript(config()?.cluster.traffic || "");
      setNodeSelector(config()?.cluster.node_selector || "");
    }
  });

  async function handleUpdateTraffic() {
    setSaving(true);
    try {
      const resp = await updateGlobalTrafficScript(script());
      setLint(resp.lint);
      addToast({
        level: "success",
        description: t("general.actions.save.status.success")!,
        duration: 5000,
      });
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.save.status.fail")!);
    }
    setSaving(false);
  }

  async function handleDeleteTraffic() {
    setSaving(true);
    try {
      await deleteGlobalTrafficScript();
      setLint(null);
      setScript("");
      addToast({
        level: "success",
        description: t("general.actions.delete.status.success")!,
        duration: 5000,
      });
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.delete.status.fail")!);
    }
    setSaving(false);
  }

  async function handleUpdateNodeSelector() {
    setSaving(true);
    try {
      await updateDefaultNodeSelector(nodeSelector());
      addToast({
        level: "success",
        description: t("general.actions.save.status.success")!,
        duration: 5000,
      });
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.save.status.fail")!);
    }
    setSaving(false);
  }

  async function handleDeleteNodeSelector() {
    setSaving(true);
    try {
      await deleteDefaultNodeSelector();
      addToast({
        level: "success",
        description: t("general.actions.delete.status.success")!,
        duration: 5000,
      });
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.delete.status.fail")!);
    }
    setSaving(false);
  }
  return (
    <>
      <Title page={t("traffic.title")} route="/admin/traffic" />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6 relative">
        <div class="flex-1 flex flex-col w-full">
          <h2 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--cloud-flow-20-regular] w-5 h-5" />
            <span class="flex-1 text-start">{t("traffic.nodeSelector")}</span>
          </h2>
          <div class="flex flex-row space-x-2 py-2 items-center">
            <span class="text-primary">ret.sh.cn/workload = </span>
            <Input size="sm" class="flex-1" value={nodeSelector()} onInput={(e) => setNodeSelector(e.target.value)} />
            <Button size="sm" level="primary" onClick={handleUpdateNodeSelector} loading={saving()}>
              {t("general.actions.save.title")}
            </Button>
            <Show when={config()?.cluster.node_selector}>
              <Popover
                level="error"
                ghost
                size="sm"
                square
                btnContent={<span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5" />}
              >
                <Card contentClass="p-2 flex flex-col space-y-2 max-w-96">
                  <span class="inline-block space-x-2">
                    <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                    <span>{t("general.actions.delete.message")}</span>
                  </span>
                  <Button level="primary" size="sm" class="self-end" onClick={handleDeleteNodeSelector}>
                    {t("general.actions.yes.title")}
                  </Button>
                </Card>
              </Popover>
            </Show>
          </div>
          <Divider />
          <h2 class="h-12 shrink-0 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="shrink-0 icon-[fluent--cloud-flow-20-regular] w-5 h-5" />
            <span class="flex-1 flex items-center justify-start space-x-2">
              <span>{t("traffic.title")}</span>
              <span class="opacity-60">$GLOBAL/traffic.rx</span>
            </span>
            <Select
              class="w-60 hidden lg:flex"
              placeholder={t("traffic.preset.title")!}
              size="sm"
              items={[
                {
                  label: t("traffic.preset.singleNodeDirectScript")!,
                  value: "single-node-direct",
                  icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
                },
                {
                  label: t("traffic.preset.multiNodeDirectScript")!,
                  value: "multi-node-direct",
                  icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
                },
              ]}
              onValueChange={(e) => {
                setPreset((e.value.at(0) as PresetTraffic) || null);
              }}
            />
            <Button size="sm" level="primary" onClick={handleUpdateTraffic} loading={saving()}>
              {t("general.actions.save.title")}
            </Button>
            <Show when={config()?.cluster.traffic}>
              <Popover
                level="error"
                ghost
                size="sm"
                square
                btnContent={<span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5" />}
              >
                <Card contentClass="p-2 flex flex-col space-y-2 max-w-96">
                  <span class="inline-block space-x-2">
                    <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5 text-warning align-middle" />
                    <span>{t("general.actions.delete.message")}</span>
                  </span>
                  <Button level="primary" size="sm" class="self-end" onClick={handleDeleteTraffic}>
                    {t("general.actions.yes.title")}
                  </Button>
                </Card>
              </Popover>
            </Show>
          </h2>
          <EditorBare
            class="w-full h-full"
            lineNumbers
            lang="rust"
            value={script()}
            lints={lint() ?? []}
            onValueChanged={(e) => {
              setScript(e);
            }}
          />
          <footer class="min-h-12 border-t border-t-layer-content/10 flex flex-col lg:flex-row flex-wrap justify-start space-x-2 items-center gap-y-2 py-2">
            <span class="text-primary icon-[fluent--info-16-regular]" />
            <span class="text-primary">{lint()?.filter((v) => v.kind === "info").length ?? 0}</span>
            <span class="text-warning icon-[fluent--warning-16-regular]" />
            <span class="text-warning">{lint()?.filter((v) => v.kind === "warning").length ?? 0}</span>
            <span class="text-error icon-[fluent--warning-16-regular]" />
            <span class="text-error">{lint()?.filter((v) => v.kind === "error").length ?? 0}</span>
            <div class="flex-1" />
            <a href="https://rune-rs.github.io/" class="text-primary hover:underline">
              Rune Grammar <span class="icon-[fluent--open-12-regular]" />
            </a>
            <span>&nbsp;&nbsp;</span>
            <a href="https://github.com/ret2shell/ret2script" class="text-primary hover:underline">
              Ret2Script <span class="icon-[fluent--open-12-regular]" />
            </a>
          </footer>
        </div>
      </div>
    </>
  );
}
