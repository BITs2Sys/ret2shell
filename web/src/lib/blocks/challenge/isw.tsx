// BITs2CTF fork: ISW (Internal Security Warfare) per-challenge panel — player view
// + admin isw.toml editor, mirroring the koh/fix panel structure.
import { handleHttpError } from "@api";
import { useGame } from "@api/game";
import { getMyRangeVpn, useChallengeIsw, useDeleteChallengeIswMutation, useUpdateChallengeIswMutation } from "@api/isw";
import type { IswConfig } from "@models/isw";
import { isAdminOfGame } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Input from "@widgets/input";
import { createEffect, createSignal, Show, untrack } from "solid-js";
import type { ChallengeWidgetProps } from ".";

function defaultConfig(): IswConfig {
  return {
    enabled: true,
    range_template: "",
    vm: "",
    guest_path: "",
    owner: null,
    mode: "0644",
    rotate: false,
  };
}

function normalizeConfig(config: IswConfig | null): IswConfig {
  return {
    ...defaultConfig(),
    ...(config ?? {}),
    owner: config?.owner || null,
    mode: config?.mode || "0644",
  };
}

function AdminIswPanel(props: { gameId: number; challengeId: number; config: IswConfig | null; onDone: () => void }) {
  const [config, setConfig] = createSignal<IswConfig>(normalizeConfig(props.config));
  createEffect(() => {
    const remote = props.config;
    untrack(() => setConfig(normalizeConfig(remote)));
  });

  const updateMutation = useUpdateChallengeIswMutation({ onSuccess: () => props.onDone() });
  const deleteMutation = useDeleteChallengeIswMutation({ onSuccess: () => props.onDone() });

  return (
    <Card contentClass="p-3 flex flex-col space-y-3">
      <header class="min-h-10 flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--shield-20-regular] w-5 h-5" />
        <span class="font-bold flex-1">{t("challenge.isw.admin")}</span>
        <Checkbox
          checked={config().enabled}
          onChange={() => setConfig((current) => ({ ...current, enabled: !current.enabled }))}
        >
          <span>{t("challenge.isw.form.enabled")}</span>
        </Checkbox>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          title={t("challenge.isw.form.rangeTemplate")}
          value={config().range_template}
          onInput={(e) => setConfig((c) => ({ ...c, range_template: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.isw.form.vm")}
          value={config().vm}
          onInput={(e) => setConfig((c) => ({ ...c, vm: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.isw.form.guestPath")}
          value={config().guest_path}
          onInput={(e) => setConfig((c) => ({ ...c, guest_path: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.isw.form.owner")}
          value={config().owner ?? ""}
          onInput={(e) => setConfig((c) => ({ ...c, owner: e.currentTarget.value || null }))}
        />
        <Input
          title={t("challenge.isw.form.mode")}
          value={config().mode}
          onInput={(e) => setConfig((c) => ({ ...c, mode: e.currentTarget.value }))}
        />
      </div>
      <Checkbox
        checked={config().rotate}
        onChange={() => setConfig((current) => ({ ...current, rotate: !current.rotate }))}
      >
        <span>{t("challenge.isw.form.rotate")}</span>
      </Checkbox>
      <div class="flex flex-row gap-2">
        <Button
          onClick={() =>
            updateMutation.mutate({
              game_id: props.gameId,
              challenge_id: props.challengeId,
              config: config(),
            })
          }
        >
          {t("challenge.isw.form.save")}
        </Button>
        <Button onClick={() => deleteMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}>
          {t("challenge.isw.actions.reset")}
        </Button>
      </div>
    </Card>
  );
}

export default function Isw(props: ChallengeWidgetProps) {
  const game = useGame({ id: () => props.gameId });
  const isw = useChallengeIsw({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });

  async function downloadVpn() {
    try {
      const config = await getMyRangeVpn(props.gameId);
      const url = URL.createObjectURL(new Blob([config], { type: "text/plain" }));
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `range-${props.gameId}.conf`;
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      handleHttpError(err as Error, t("challenge.isw.player.vpn"));
    }
  }

  return (
    <div class="flex flex-col space-y-3">
      <Show
        when={isw.data?.enabled}
        fallback={<Card contentClass="p-3 text-sm opacity-70">{t("challenge.isw.disabled")}</Card>}
      >
        <Card contentClass="p-3 flex flex-col space-y-2">
          <span class="font-bold">{t("challenge.isw.player.myRange")}</span>
          <span class="text-sm opacity-80">{t("challenge.isw.player.noRange")}</span>
          <div class="flex flex-row gap-2">
            <Button onClick={() => downloadVpn()}>
              <span class="shrink-0 icon-[fluent--arrow-download-20-regular] w-5 h-5" />
              <span>{t("challenge.isw.player.download")}</span>
            </Button>
          </div>
        </Card>
      </Show>
      <Show when={isAdminOfGame(game.data)}>
        <AdminIswPanel
          gameId={props.gameId}
          challengeId={props.challengeId}
          config={isw.data ?? null}
          onDone={() => isw.refetch()}
        />
      </Show>
    </div>
  );
}
