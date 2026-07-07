// BITs2CTF fork: AWDP (Attack-and-Defense Plus) per-challenge panel — Jeopardy-style
// solve/fix with a persistent per-round bonus. Player status + admin awdp.toml editor,
// mirroring the koh/isw panel structure.
import { useChallengeAwdp, useDeleteChallengeAwdpMutation, useUpdateChallengeAwdpMutation } from "@api/awd";
import { useGame } from "@api/game";
import type { AwdpConfig, AwdpMode } from "@models/awd";
import { isAdminOfGame } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Input from "@widgets/input";
import Select from "@widgets/select";
import Tag from "@widgets/tag";
import { createEffect, createSignal, Show, untrack } from "solid-js";
import type { ChallengeWidgetProps } from ".";

function defaultConfig(): AwdpConfig {
  return {
    enabled: true,
    mode: "solve",
    round_secs: 300,
    total_rounds: 0,
  };
}

function normalizeConfig(config: AwdpConfig | null): AwdpConfig {
  return {
    ...defaultConfig(),
    ...(config ?? {}),
    mode: config?.mode ?? "solve",
    round_secs: config?.round_secs || 300,
    total_rounds: config?.total_rounds ?? 0,
  };
}

function AdminAwdpPanel(props: { gameId: number; challengeId: number; config: AwdpConfig | null; onDone: () => void }) {
  const [config, setConfig] = createSignal<AwdpConfig>(normalizeConfig(props.config));
  createEffect(() => {
    const remote = props.config;
    untrack(() => setConfig(normalizeConfig(remote)));
  });

  const updateMutation = useUpdateChallengeAwdpMutation({ onSuccess: () => props.onDone() });
  const deleteMutation = useDeleteChallengeAwdpMutation({ onSuccess: () => props.onDone() });

  return (
    <Card contentClass="p-3 flex flex-col space-y-3">
      <header class="min-h-10 flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--shield-badge-20-regular] w-5 h-5" />
        <span class="font-bold flex-1">{t("challenge.awdp.admin")}</span>
        <Checkbox
          checked={config().enabled}
          onChange={() => setConfig((current) => ({ ...current, enabled: !current.enabled }))}
        >
          <span>{t("challenge.awdp.form.enabled")}</span>
        </Checkbox>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Select
          label={t("challenge.awdp.form.mode")}
          value={[config().mode]}
          items={[
            {
              label: t("challenge.awdp.mode.solve"),
              value: "solve",
              icon: "icon-[fluent--flag-20-regular] w-5 h-5",
            },
            {
              label: t("challenge.awdp.mode.fix"),
              value: "fix",
              icon: "icon-[fluent--wrench-20-regular] w-5 h-5",
            },
          ]}
          onValueChange={(e) => setConfig((current) => ({ ...current, mode: (e.value.at(0) as AwdpMode) || "solve" }))}
        />
        <Input
          type="number"
          title={t("challenge.awdp.form.roundSecs")}
          value={config().round_secs}
          onInput={(e) => setConfig((current) => ({ ...current, round_secs: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          type="number"
          title={t("challenge.awdp.form.totalRounds")}
          value={config().total_rounds}
          onInput={(e) => setConfig((current) => ({ ...current, total_rounds: Number(e.currentTarget.value) || 0 }))}
        />
      </div>
      <span class="text-xs opacity-60">{t("challenge.awdp.form.hint")}</span>
      <div class="flex flex-row gap-2">
        <Button
          loading={updateMutation.isPending}
          onClick={() =>
            updateMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId, config: config() })
          }
        >
          {t("challenge.awdp.form.save")}
        </Button>
        <Button ghost onClick={() => deleteMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}>
          {t("challenge.awdp.actions.reset")}
        </Button>
      </div>
    </Card>
  );
}

export default function Awdp(props: ChallengeWidgetProps) {
  const game = useGame({ id: () => props.gameId });
  const awdp = useChallengeAwdp({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });

  return (
    <div class="flex flex-col space-y-3">
      <Show
        when={awdp.data?.config?.enabled}
        fallback={<Card contentClass="p-3 text-sm opacity-70">{t("challenge.awdp.disabled")}</Card>}
      >
        <Card contentClass="p-3 flex flex-col space-y-2">
          <div class="flex flex-row flex-wrap items-center gap-2">
            <span class="shrink-0 icon-[fluent--shield-badge-20-regular] w-5 h-5" />
            <span class="font-bold flex-1">{t("challenge.awdp.player.title")}</span>
            <Tag>{t(`challenge.awdp.mode.${awdp.data?.config?.mode ?? "solve"}`)}</Tag>
            <Show when={awdp.data?.solved}>
              <Tag class="text-success">{t("challenge.awdp.player.secured")}</Tag>
            </Show>
          </div>
          <div class="grid grid-cols-fit-xs gap-2 text-sm">
            <div class="flex flex-col">
              <span class="opacity-60">{t("challenge.awdp.player.round")}</span>
              <span class="font-mono">{awdp.data?.round ?? 0}</span>
            </div>
            <div class="flex flex-col">
              <span class="opacity-60">{t("challenge.awdp.player.roundSecs")}</span>
              <span class="font-mono">{awdp.data?.config?.round_secs ?? 0}s</span>
            </div>
            <Show when={awdp.data?.solved && awdp.data?.solved_round != null}>
              <div class="flex flex-col">
                <span class="opacity-60">{t("challenge.awdp.player.solvedRound")}</span>
                <span class="font-mono">{awdp.data?.solved_round}</span>
              </div>
            </Show>
          </div>
          <span class="text-xs opacity-70">
            {awdp.data?.config?.mode === "fix"
              ? t("challenge.awdp.player.fixHint")
              : t("challenge.awdp.player.solveHint")}
          </span>
        </Card>
      </Show>
      <Show when={isAdminOfGame(game.data)}>
        <AdminAwdpPanel
          gameId={props.gameId}
          challengeId={props.challengeId}
          config={awdp.data?.config ?? null}
          onDone={() => awdp.refetch()}
        />
      </Show>
    </div>
  );
}
