// BITs2CTF fork: AWD (Attack-and-Defense) per-challenge panel — each team gets its
// own machine; flags rotate every round and an SLA check runs. Player view shows the
// team's machine + current round; admin gets the awd.toml editor + provision/teardown
// + steal scoreboard. Mirrors the koh/fix panel structure.
import { toastError } from "@api";
import {
  getAwdScoreboard,
  useChallengeAwd,
  useDeleteChallengeAwdMutation,
  useProvisionAwdMutation,
  useTeardownAwdMutation,
  useUpdateChallengeAwdMutation,
} from "@api/awd";
import { useGame } from "@api/game";
import type { AwdConfig, AwdSteal } from "@models/awd";
import type { ChallengeImage } from "@models/challenge";
import { isAdminOfGame } from "@storage/game";
import { t } from "@storage/theme";
import { useQuery } from "@tanstack/solid-query";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import ClipboardBtn from "@widgets/clipboard-btn";
import Input from "@widgets/input";
import Tag from "@widgets/tag";
import { createEffect, createMemo, createSignal, For, Show, untrack } from "solid-js";
import type { ChallengeWidgetProps } from ".";

function defaultImage(): ChallengeImage {
  return {
    name: "",
    tag: "latest",
    cpu: 1,
    cpu_req: 0.1,
    mem: "512Mi",
    mem_req: "128Mi",
    storage: "1Gi",
    storage_req: "256Mi",
    port: 80,
    protocol: "tcp",
    app_protocol: null,
    service_type: "tcp",
    description: null,
    restricted: null,
  };
}

function defaultConfig(): AwdConfig {
  return {
    enabled: true,
    round_secs: 300,
    internet: false,
    restricted: null,
    privileged: null,
    image: defaultImage(),
    pull_secret: null,
    flag_path: "/flag",
    check_command: null,
    attack_reward: 100,
    defense_reward: 50,
    sla_reward: 30,
    timeout_secs: 10,
  };
}

function normalizeConfig(config: AwdConfig | null): AwdConfig {
  return {
    ...defaultConfig(),
    ...(config ?? {}),
    image: config?.image ? { ...defaultImage(), ...config.image } : defaultImage(),
    flag_path: config?.flag_path || "/flag",
    pull_secret: config?.pull_secret || null,
    check_command: config?.check_command ?? null,
  };
}

function AdminAwdPanel(props: { gameId: number; challengeId: number; config: AwdConfig | null; onDone: () => void }) {
  const [config, setConfig] = createSignal<AwdConfig>(normalizeConfig(props.config));
  const [checkText, setCheckText] = createSignal(JSON.stringify(props.config?.check_command ?? [], null, 2));
  createEffect(() => {
    const remote = props.config;
    untrack(() => {
      setConfig(normalizeConfig(remote));
      setCheckText(JSON.stringify(remote?.check_command ?? [], null, 2));
    });
  });

  const updateMutation = useUpdateChallengeAwdMutation({ onSuccess: () => props.onDone() });
  const deleteMutation = useDeleteChallengeAwdMutation({ onSuccess: () => props.onDone() });
  const provisionMutation = useProvisionAwdMutation({ onSuccess: () => props.onDone() });
  const teardownMutation = useTeardownAwdMutation({ onSuccess: () => props.onDone() });

  function updateImage(patch: Partial<ChallengeImage>) {
    setConfig((current) => ({ ...current, image: { ...current.image, ...patch } }));
  }

  function save() {
    let checkCommand: string[] | null = null;
    const raw = checkText().trim();
    if (raw.length > 0) {
      try {
        const parsed = JSON.parse(raw);
        if (!Array.isArray(parsed) || parsed.some((v) => typeof v !== "string")) {
          throw new Error("check command must be a string array");
        }
        checkCommand = parsed.length > 0 ? parsed : null;
      } catch (err) {
        toastError(`${t("challenge.awd.form.checkCommand.invalid")}: ${err}`);
        return;
      }
    }
    updateMutation.mutate({
      game_id: props.gameId,
      challenge_id: props.challengeId,
      config: { ...config(), check_command: checkCommand },
    });
  }

  return (
    <Card contentClass="p-3 flex flex-col space-y-3">
      <header class="min-h-10 flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--target-arrow-20-regular] w-5 h-5" />
        <span class="font-bold flex-1">{t("challenge.awd.admin")}</span>
        <Checkbox
          checked={config().enabled}
          onChange={() => setConfig((current) => ({ ...current, enabled: !current.enabled }))}
        >
          <span>{t("challenge.awd.form.enabled")}</span>
        </Checkbox>
      </header>

      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          type="number"
          title={t("challenge.awd.form.roundSecs")}
          value={config().round_secs}
          onInput={(e) => setConfig((c) => ({ ...c, round_secs: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          title={t("challenge.awd.form.flagPath")}
          value={config().flag_path}
          onInput={(e) => setConfig((c) => ({ ...c, flag_path: e.currentTarget.value }))}
        />
        <Input
          type="number"
          title={t("challenge.awd.form.timeoutSecs")}
          value={config().timeout_secs}
          onInput={(e) => setConfig((c) => ({ ...c, timeout_secs: Number(e.currentTarget.value) || 1 }))}
        />
      </div>

      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          type="number"
          title={t("challenge.awd.form.attackReward")}
          value={config().attack_reward}
          onInput={(e) => setConfig((c) => ({ ...c, attack_reward: Number(e.currentTarget.value) || 0 }))}
        />
        <Input
          type="number"
          title={t("challenge.awd.form.defenseReward")}
          value={config().defense_reward}
          onInput={(e) => setConfig((c) => ({ ...c, defense_reward: Number(e.currentTarget.value) || 0 }))}
        />
        <Input
          type="number"
          title={t("challenge.awd.form.slaReward")}
          value={config().sla_reward}
          onInput={(e) => setConfig((c) => ({ ...c, sla_reward: Number(e.currentTarget.value) || 0 }))}
        />
      </div>

      <div class="flex flex-row flex-wrap gap-3">
        <Checkbox checked={config().internet} onChange={() => setConfig((c) => ({ ...c, internet: !c.internet }))}>
          <span>{t("challenge.awd.form.internet")}</span>
        </Checkbox>
        <Checkbox
          checked={config().privileged ?? false}
          onChange={() => setConfig((c) => ({ ...c, privileged: !(c.privileged ?? false) }))}
        >
          <span>{t("challenge.awd.form.privileged")}</span>
        </Checkbox>
      </div>

      <header class="min-h-10 flex flex-row gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--box-20-regular] w-5 h-5" />
        <span class="font-bold">{t("challenge.awd.machine")}</span>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          title={t("challenge.instance.image.form.containerName.label")}
          value={config().image.name}
          onInput={(e) => updateImage({ name: e.currentTarget.value })}
        />
        <Input
          title={t("challenge.instance.image.form.tag.label")}
          value={config().image.tag}
          onInput={(e) => updateImage({ tag: e.currentTarget.value })}
        />
        <Input
          type="number"
          title={t("challenge.awd.form.port")}
          value={config().image.port ?? 0}
          onInput={(e) => updateImage({ port: Number(e.currentTarget.value) || null })}
        />
        <Input
          type="number"
          title={t("challenge.instance.image.form.service.cpu.label")}
          value={config().image.cpu}
          onInput={(e) => updateImage({ cpu: Number(e.currentTarget.value) || 1 })}
        />
        <Input
          title={t("challenge.instance.image.form.service.mem.label")}
          value={config().image.mem}
          onInput={(e) => updateImage({ mem: e.currentTarget.value })}
        />
        <Input
          title={t("challenge.awd.form.pullSecret")}
          value={config().pull_secret ?? ""}
          onInput={(e) => setConfig((c) => ({ ...c, pull_secret: e.currentTarget.value || null }))}
        />
      </div>

      <label class="flex flex-col space-y-1">
        <span class="label">{t("challenge.awd.form.checkCommand.label")}</span>
        <textarea
          class="input min-h-24 font-mono text-sm"
          placeholder='["/bin/sh", "-c", "curl -fs http://localhost/health"]'
          value={checkText()}
          onInput={(e) => setCheckText(e.currentTarget.value)}
        />
        <span class="text-xs opacity-60">{t("challenge.awd.form.checkCommand.hint")}</span>
      </label>

      <div class="flex flex-row flex-wrap justify-between gap-2">
        <div class="flex flex-row gap-2">
          <Button
            loading={provisionMutation.isPending}
            onClick={() => provisionMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          >
            <span class="shrink-0 icon-[fluent--rocket-20-regular] w-5 h-5" />
            <span>{t("challenge.awd.actions.provision")}</span>
          </Button>
          <Button
            ghost
            loading={teardownMutation.isPending}
            onClick={() => teardownMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          >
            <span class="shrink-0 icon-[fluent--plug-disconnected-20-regular] w-5 h-5" />
            <span>{t("challenge.awd.actions.teardown")}</span>
          </Button>
        </div>
        <div class="flex flex-row gap-2">
          <Button
            level="error"
            loading={deleteMutation.isPending}
            onClick={() => deleteMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId })}
          >
            {t("general.actions.delete.title")}
          </Button>
          <Button level="primary" onClick={save} loading={updateMutation.isPending}>
            {t("general.actions.save.title")}
          </Button>
        </div>
      </div>
    </Card>
  );
}

function AwdScoreboard(props: { gameId: number; challengeId: number }) {
  const scoreboard = useQuery(() => ({
    queryKey: ["game", props.gameId, "challenge", props.challengeId, "awd", "scoreboard"],
    queryFn: async () => (await getAwdScoreboard(props.gameId, props.challengeId)) as AwdSteal[],
    refetchInterval: 15_000,
  }));
  return (
    <Card contentClass="p-3 flex flex-col space-y-2">
      <span class="font-bold">{t("challenge.awd.scoreboard.title")}</span>
      <Show
        when={(scoreboard.data?.length ?? 0) > 0}
        fallback={<span class="text-sm opacity-60">{t("challenge.awd.scoreboard.empty")}</span>}
      >
        <div class="flex flex-col divide-y divide-layer-content/10">
          <For each={scoreboard.data}>
            {(steal) => (
              <div class="flex flex-row items-center gap-2 py-1 text-sm">
                <Tag class="font-mono">#{steal.round}</Tag>
                <span class="flex-1 truncate">
                  {steal.attacker_name ?? `team ${steal.attacker_team_id}`}
                  <span class="opacity-50"> → </span>
                  <span class="opacity-70">team {steal.victim_team_id}</span>
                </span>
                <span class="font-mono text-success">+{steal.score}</span>
              </div>
            )}
          </For>
        </div>
      </Show>
    </Card>
  );
}

export default function Awd(props: ChallengeWidgetProps) {
  const game = useGame({ id: () => props.gameId });
  const awd = useChallengeAwd({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  const admin = createMemo(() => isAdminOfGame(game.data));

  return (
    <div class="flex flex-col space-y-3">
      <Show
        when={awd.data?.config?.enabled}
        fallback={<Card contentClass="p-3 text-sm opacity-70">{t("challenge.awd.disabled")}</Card>}
      >
        <Card contentClass="p-3 flex flex-col space-y-2">
          <div class="flex flex-row flex-wrap items-center gap-2">
            <span class="shrink-0 icon-[fluent--target-arrow-20-regular] w-5 h-5" />
            <span class="font-bold flex-1">{t("challenge.awd.player.title")}</span>
            <Tag class="font-mono">
              {t("challenge.awd.player.round")} {awd.data?.round ?? 0}
            </Tag>
          </div>
          <Show
            when={awd.data?.instance}
            fallback={<span class="text-sm opacity-70">{t("challenge.awd.player.noMachine")}</span>}
          >
            <div class="flex flex-row flex-wrap items-center gap-2 text-sm">
              <span class="opacity-60">{t("challenge.awd.player.myMachine")}</span>
              <Show when={awd.data?.instance?.address} fallback={<span class="opacity-50">—</span>}>
                <span class="font-mono">{awd.data?.instance?.address}</span>
                <ClipboardBtn size="sm" square value={awd.data?.instance?.address ?? ""} />
              </Show>
              <Tag class={awd.data?.instance?.status === "running" ? "text-success" : "text-warning"}>
                {awd.data?.instance?.status}
              </Tag>
            </div>
          </Show>
          <span class="text-xs opacity-70">{t("challenge.awd.player.hint")}</span>
        </Card>
        <AwdScoreboard gameId={props.gameId} challengeId={props.challengeId} />
      </Show>
      <Show when={admin()}>
        <AdminAwdPanel
          gameId={props.gameId}
          challengeId={props.challengeId}
          config={awd.data?.config ?? null}
          onDone={() => awd.refetch()}
        />
      </Show>
    </div>
  );
}
