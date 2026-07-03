import { inflyClient } from "@api";
import {
  useChallengeKoh,
  useCheckKohOnceMutation,
  useDeleteChallengeKohMutation,
  useKohEvents,
  useKohScoreboard,
  useStartKohHillMutation,
  useStopKohHillMutation,
  useUpdateChallengeKohMutation,
} from "@api/challenge";
import { useGame } from "@api/game";
import { getWsrxLink } from "@lib/wsrx";
import type { KohConfig, KohEvent, KohMode } from "@models/challenge";
import { isAdminOfGame } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import ClipboardBtn from "@widgets/clipboard-btn";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import Select from "@widgets/select";
import Tag from "@widgets/tag";
import { createEffect, createMemo, createSignal, For, Match, Show, Switch, untrack } from "solid-js";
import type { ChallengeWidgetProps } from ".";

function defaultConfig(): KohConfig {
  return {
    enabled: true,
    mode: "agent_http",
    interval_secs: 10,
    round_secs: 60,
    total_rounds: 0,
    reward: 1,
    rank_count: 1,
    rank_percentages: [100],
    status_url: null,
    status_path: "/status",
    api_key: null,
    agent_port: null,
    target_port: null,
    timeout_secs: 3,
    auto_start: true,
    elo: null,
  };
}

function normalizeConfig(config: KohConfig | null): KohConfig {
  return {
    ...defaultConfig(),
    ...(config ?? {}),
    mode: config?.mode ?? "agent_http",
    status_url: config?.status_url || null,
    api_key: config?.api_key || null,
    agent_port: config?.agent_port || null,
    target_port: config?.target_port || null,
    status_path: config?.status_path || "/status",
    rank_count: config?.rank_count || 1,
    rank_percentages: config?.rank_percentages?.length ? config.rank_percentages : [100],
  };
}

function parsePercentages(value: string) {
  return value
    .split(",")
    .map((item) => Number.parseInt(item.trim(), 10))
    .filter((item) => Number.isFinite(item));
}

function eventLevel(event: KohEvent): "info" | "success" | "warning" | "error" | "layer-content" {
  switch (event.status) {
    case "captured":
    case "awarded":
    case "rank_awarded":
    case "round_awarded":
      return "success";
    case "held":
    case "empty":
    case "pending_round":
    case "completed":
      return "info";
    case "unknown_identifier":
    case "rank_skipped":
      return "warning";
    default:
      return "error";
  }
}

function AdminKohPanel(props: { gameId: number; challengeId: number; config: KohConfig | null; onDone: () => void }) {
  const [config, setConfig] = createSignal<KohConfig>(normalizeConfig(props.config));

  createEffect(() => {
    const remote = props.config;
    untrack(() => setConfig(normalizeConfig(remote)));
  });

  const updateMutation = useUpdateChallengeKohMutation({ onSuccess: () => props.onDone() });
  const deleteMutation = useDeleteChallengeKohMutation({ onSuccess: () => props.onDone() });
  const startMutation = useStartKohHillMutation({ onSuccess: () => props.onDone() });
  const stopMutation = useStopKohHillMutation({ onSuccess: () => props.onDone() });
  const checkMutation = useCheckKohOnceMutation({ onSuccess: () => props.onDone() });

  function save() {
    updateMutation.mutate({
      game_id: props.gameId,
      challenge_id: props.challengeId,
      config: config(),
    });
  }

  return (
    <Card contentClass="p-3 flex flex-col space-y-3">
      <header class="min-h-10 flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--crown-20-regular] w-5 h-5" />
        <span class="font-bold flex-1">{t("challenge.koh.admin")}</span>
        <Checkbox
          checked={config().enabled}
          onChange={() => setConfig((current) => ({ ...current, enabled: !current.enabled }))}
        >
          <span>{t("challenge.koh.form.enabled")}</span>
        </Checkbox>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Select
          label={t("challenge.koh.form.mode")}
          value={[config().mode]}
          items={[
            {
              label: t("challenge.koh.mode.agentHttp"),
              value: "agent_http",
              icon: "icon-[fluent--crown-20-regular] w-5 h-5",
            },
            {
              label: t("challenge.koh.mode.roundRankHttp"),
              value: "round_rank_http",
              icon: "icon-[fluent--data-bar-vertical-ascending-20-regular] w-5 h-5",
            },
            {
              label: t("challenge.koh.mode.gameElo"),
              value: "game_elo",
              icon: "icon-[fluent--games-20-regular] w-5 h-5",
              disabled: true,
            },
          ]}
          onValueChange={(e) => setConfig((current) => ({ ...current, mode: (e.value.at(0) as KohMode) || "agent_http" }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.interval")}
          value={config().interval_secs}
          onInput={(e) => setConfig((current) => ({ ...current, interval_secs: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.reward")}
          value={config().reward}
          onInput={(e) => setConfig((current) => ({ ...current, reward: Number(e.currentTarget.value) || 0 }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.roundSecs")}
          value={config().round_secs}
          onInput={(e) => setConfig((current) => ({ ...current, round_secs: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.totalRounds")}
          value={config().total_rounds}
          onInput={(e) => setConfig((current) => ({ ...current, total_rounds: Number(e.currentTarget.value) || 0 }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.rankCount")}
          value={config().rank_count}
          onInput={(e) =>
            setConfig((current) => ({
              ...current,
              rank_count: Number(e.currentTarget.value) || 1,
            }))
          }
        />
        <Input
          title={t("challenge.koh.form.rankPercentages")}
          value={config().rank_percentages.join(",")}
          onInput={(e) =>
            setConfig((current) => ({
              ...current,
              rank_percentages: parsePercentages(e.currentTarget.value),
            }))
          }
        />
        <Input
          type="number"
          title={t("challenge.koh.form.timeout")}
          value={config().timeout_secs}
          onInput={(e) => setConfig((current) => ({ ...current, timeout_secs: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          title={t("challenge.koh.form.statusPath")}
          value={config().status_path}
          onInput={(e) => setConfig((current) => ({ ...current, status_path: e.currentTarget.value || "/status" }))}
        />
        <Input
          title={t("challenge.koh.form.statusUrl")}
          value={config().status_url ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, status_url: e.currentTarget.value || null }))}
        />
        <Input
          type="password"
          title={t("challenge.koh.form.apiKey")}
          value={config().api_key ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, api_key: e.currentTarget.value || null }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.agentPort")}
          value={config().agent_port ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, agent_port: Number(e.currentTarget.value) || null }))}
        />
        <Input
          type="number"
          title={t("challenge.koh.form.targetPort")}
          value={config().target_port ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, target_port: Number(e.currentTarget.value) || null }))}
        />
      </div>
      <Checkbox
        checked={config().auto_start}
        onChange={() => setConfig((current) => ({ ...current, auto_start: !current.auto_start }))}
      >
        <span>{t("challenge.koh.form.autoStart")}</span>
      </Checkbox>
      <div class="flex flex-row flex-wrap justify-end gap-2">
        <Button
          onClick={() => checkMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          loading={checkMutation.isPending}
          disabled={checkMutation.isPending}
        >
          <span class="shrink-0 icon-[fluent--flash-checkmark-20-regular] w-5 h-5" />
          <span>{t("challenge.koh.actions.check")}</span>
        </Button>
        <Button
          onClick={() => startMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          loading={startMutation.isPending}
          disabled={startMutation.isPending}
        >
          <span class="shrink-0 icon-[fluent--play-20-regular] w-5 h-5" />
          <span>{t("challenge.koh.actions.start")}</span>
        </Button>
        <Button
          onClick={() => stopMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          loading={stopMutation.isPending}
          disabled={stopMutation.isPending}
        >
          <span class="shrink-0 icon-[fluent--record-stop-20-regular] w-5 h-5" />
          <span>{t("challenge.koh.actions.stop")}</span>
        </Button>
        <Button
          level="error"
          onClick={() => deleteMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          loading={deleteMutation.isPending}
          disabled={deleteMutation.isPending}
        >
          {t("general.actions.delete.title")}
        </Button>
        <Button level="primary" onClick={save} loading={updateMutation.isPending} disabled={updateMutation.isPending}>
          {t("general.actions.save.title")}
        </Button>
      </div>
    </Card>
  );
}

export default function Koh(props: ChallengeWidgetProps) {
  const game = useGame({ id: () => props.gameId });
  const koh = useChallengeKoh({ game_id: () => props.gameId, challenge_id: () => props.challengeId });
  const events = useKohEvents({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
    enabled: () => !!koh.data?.config?.enabled,
  });
  const scoreboard = useKohScoreboard({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
    enabled: () => !!koh.data?.config?.enabled,
  });
  const ownsHill = createMemo(
    () => !!koh.data?.identifier && koh.data.state?.current_team_id === koh.data.identifier.team_id
  );

  function refresh() {
    koh.refetch();
    events.refetch();
    scoreboard.refetch();
    inflyClient.invalidateQueries({ queryKey: ["game", props.gameId, "team"] });
  }

  return (
    <div class="flex-1 flex flex-col space-y-3 p-3 lg:p-6">
      <Show when={koh.isLoading}>
        <LoadingTips />
      </Show>
      <Show when={koh.data?.config?.enabled} fallback={<Card contentClass="p-3">{t("challenge.koh.disabled")}</Card>}>
        <div class="grid grid-cols-1 xl:grid-cols-2 gap-3">
          <Card contentClass="p-3 flex flex-col gap-3">
            <header class="min-h-10 flex flex-row items-center gap-2 border-b border-b-layer-content/10 pb-2">
              <span class="shrink-0 icon-[fluent--crown-20-regular] w-5 h-5" />
              <span class="font-bold flex-1">{t("challenge.koh.title")}</span>
              <Tag level={koh.data?.target?.state === "Running" ? "success" : "warning"}>
                <span>{koh.data?.target?.state ?? t("challenge.koh.status.idle")}</span>
              </Tag>
            </header>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
              <Card contentClass="p-2 flex flex-col gap-2">
                <span class="text-sm opacity-70">{t("challenge.koh.identifier")}</span>
                <Show
                  when={koh.data?.identifier?.identifier}
                  fallback={<span class="font-bold opacity-60">{t("challenge.koh.noIdentifier")}</span>}
                >
                  <ClipboardBtn
                    size="sm"
                    title={t("general.actions.copy.title")}
                    value={koh.data!.identifier!.identifier}
                    label={koh.data!.identifier!.identifier}
                  />
                </Show>
              </Card>
              <Card contentClass="p-2 flex flex-col gap-2">
                <span class="text-sm opacity-70">{t("challenge.koh.owner")}</span>
                <div class="flex flex-row gap-2 items-center">
                  <Tag level={ownsHill() ? "success" : "info"}>
                    <span>{koh.data?.state?.current_identifier ?? t("challenge.koh.noOwner")}</span>
                  </Tag>
                  <Show when={ownsHill()}>
                    <span class="text-success font-bold">{t("challenge.koh.owned")}</span>
                  </Show>
                </div>
              </Card>
            </div>
            <Show when={koh.data?.target}>
              <Card contentClass="p-2 flex flex-col gap-2">
                <span class="text-sm opacity-70">{t("challenge.koh.target")}</span>
                <For each={koh.data?.target?.exposed_ports ?? []}>
                  {(port) => (
                    <ClipboardBtn
                      size="sm"
                      title={t("challenge.instance.actions.copy.title")}
                      value={port.address}
                      label={`${port.name}: ${port.address}`}
                    />
                  )}
                </For>
                <Show when={!koh.data?.target?.exposed_ports?.length}>
                  <div class="flex flex-row flex-wrap gap-2">
                    <For each={koh.data?.target?.ports ?? []}>
                      {(port) => (
                        <ClipboardBtn
                          size="sm"
                          title={t("wsrx.actions.copy.title")}
                          value={getWsrxLink(koh.data!.target!.traffic, port)}
                          label={`WSRX:${port}`}
                        />
                      )}
                    </For>
                  </div>
                </Show>
              </Card>
            </Show>
            <div class="flex flex-row flex-wrap gap-2">
              <Tag level="info">
                <Show
                  when={koh.data?.config?.mode === "round_rank_http"}
                  fallback={
                    <span>
                      {koh.data?.config?.reward ?? 0} pts / {koh.data?.config?.interval_secs ?? 0}s
                    </span>
                  }
                >
                  <span>
                    {koh.data?.config?.reward ?? 0} pts / {koh.data?.config?.round_secs ?? 0}s
                  </span>
                </Show>
              </Tag>
              <Show when={koh.data?.config?.mode === "round_rank_http"}>
                <Tag level="info">
                  <span>
                    top {koh.data?.config?.rank_count ?? 0}:{" "}
                    {(koh.data?.config?.rank_percentages ?? []).slice(0, koh.data?.config?.rank_count ?? 0).join("/")}
                    %
                  </span>
                </Tag>
              </Show>
              <Show when={koh.data?.config?.total_rounds}>
                <Tag level="layer-content">
                  <span>{koh.data?.config?.total_rounds} rounds</span>
                </Tag>
              </Show>
              <Tag level="layer-content">
                <span>{koh.data?.config?.mode ?? "agent_http"}</span>
              </Tag>
              <Show when={koh.data?.state?.last_checked_at}>
                <Tag level="layer-content">
                  <span>{koh.data?.state?.last_checked_at?.toFormat("MM-dd HH:mm:ss")}</span>
                </Tag>
              </Show>
              <Show when={koh.data?.state?.last_error}>
                <Tag level="error">
                  <span>{koh.data?.state?.last_error}</span>
                </Tag>
              </Show>
            </div>
          </Card>
          <Card contentClass="p-3 flex flex-col gap-3">
            <header class="min-h-10 flex flex-row items-center gap-2 border-b border-b-layer-content/10 pb-2">
              <span class="shrink-0 icon-[fluent--data-bar-vertical-20-regular] w-5 h-5" />
              <span class="font-bold flex-1">{t("challenge.koh.scoreboard")}</span>
              <Button square ghost size="sm" onClick={refresh}>
                <span class="shrink-0 icon-[fluent--arrow-clockwise-20-regular] w-5 h-5" />
              </Button>
            </header>
            <Show
              when={(scoreboard.data?.length ?? 0) > 0}
              fallback={<span class="opacity-60">{t("challenge.koh.emptyScoreboard")}</span>}
            >
              <For each={scoreboard.data}>
                {(entry, index) => (
                  <div class="h-10 flex flex-row items-center gap-2 border-b border-b-layer-content/10">
                    <span class="w-8 text-center font-bold opacity-60">{index() + 1}</span>
                    <span class="flex-1 truncate">{entry.team_name ?? `#${entry.team_id}`}</span>
                    <span class="font-bold">{entry.score} pts</span>
                  </div>
                )}
              </For>
            </Show>
          </Card>
        </div>
        <Card contentClass="p-3 flex flex-col gap-3">
          <header class="min-h-10 flex flex-row items-center gap-2 border-b border-b-layer-content/10 pb-2">
            <span class="shrink-0 icon-[fluent--history-20-regular] w-5 h-5" />
            <span class="font-bold">{t("challenge.koh.events")}</span>
          </header>
          <Show
            when={(events.data?.length ?? 0) > 0}
            fallback={<span class="opacity-60">{t("challenge.koh.emptyEvents")}</span>}
          >
            <For each={events.data}>
              {(event) => (
                <div class="min-h-10 flex flex-row flex-wrap items-center gap-2 border-b border-b-layer-content/10 py-1">
                  <Tag level={eventLevel(event)}>
                    <span>{event.status}</span>
                  </Tag>
                  <span class="font-mono text-sm">{event.identifier ?? "-"}</span>
                  <span class="flex-1 truncate">{event.team_name ?? event.message ?? ""}</span>
                  <Switch>
                    <Match when={event.score_delta > 0}>
                      <span class="font-bold text-success">+{event.score_delta} pts</span>
                    </Match>
                  </Switch>
                  <span class="opacity-60">{event.created_at.toFormat("MM-dd HH:mm:ss")}</span>
                </div>
              )}
            </For>
          </Show>
        </Card>
      </Show>
      <Show when={isAdminOfGame(game.data)}>
        <AdminKohPanel
          gameId={props.gameId}
          challengeId={props.challengeId}
          config={koh.data?.config ?? null}
          onDone={refresh}
        />
      </Show>
    </div>
  );
}
