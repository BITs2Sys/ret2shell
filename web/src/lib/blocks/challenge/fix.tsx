import { inflyClient } from "@api";
import {
  useChallengeFix,
  useDeleteChallengeFixMutation,
  useSubmitFixMutation,
  useUpdateChallengeFixMutation,
} from "@api/challenge";
import { checkSubmissionStatus, useGame } from "@api/game";
import { humanFileSize } from "@lib/utils/size";
import type { ChallengeImage, FixConfig } from "@models/challenge";
import { isAdminOfGame, isGameInProgress } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Checkbox from "@widgets/checkbox";
import Input from "@widgets/input";
import LoadingTips from "@widgets/loading-tips";
import { createEffect, createSignal, Show, untrack } from "solid-js";
import type { ChallengeWidgetProps } from ".";

function defaultTester(): ChallengeImage {
  return {
    name: "fix-tester",
    tag: "",
    cpu: 0.5,
    cpu_req: 0.01,
    mem: "128Mi",
    mem_req: "32Mi",
    storage: "512Mi",
    storage_req: "64Mi",
    port: null,
    protocol: null,
    app_protocol: null,
    service_type: null,
    description: null,
    restricted: null,
  };
}

function defaultConfig(): FixConfig {
  return {
    enabled: true,
    max_attempts: 3,
    fix_script: "fix.sh",
    upload_path: "/tmp/ret2shell-fix/submission",
    target_container: null,
    target_port: null,
    tester: defaultTester(),
    tester_command: ["/bin/sh", "-c", ". /check.sh; printf 'R2S_FIX_RESULT=%s\\n' \"$R2S_FIX_RESULT\""],
    result_env: "R2S_FIX_RESULT",
    success_value: "success",
    timeout_secs: 120,
    pull_secret: null,
  };
}

function normalizeConfig(config: FixConfig): FixConfig {
  return {
    ...config,
    target_container: config.target_container || null,
    target_port: config.target_port || null,
    tester: config.tester ? { ...defaultTester(), ...config.tester } : defaultTester(),
    tester_command: config.tester_command?.length ? config.tester_command : null,
    pull_secret: config.pull_secret || null,
  };
}

function AdminFixPanel(props: { gameId: number; challengeId: number; config: FixConfig | null; onDone: () => void }) {
  const [config, setConfig] = createSignal<FixConfig>(normalizeConfig(props.config ?? defaultConfig()));
  const [commandText, setCommandText] = createSignal(JSON.stringify(config().tester_command ?? [], null, 2));

  createEffect(() => {
    const remote = props.config;
    untrack(() => {
      setConfig(normalizeConfig(remote ?? defaultConfig()));
      setCommandText(JSON.stringify((remote ?? defaultConfig()).tester_command ?? [], null, 2));
    });
  });

  function updateTester(patch: Partial<ChallengeImage>) {
    setConfig((current) => ({
      ...current,
      tester: {
        ...defaultTester(),
        ...(current.tester ?? {}),
        ...patch,
      },
    }));
  }

  const updateMutation = useUpdateChallengeFixMutation({
    onSuccess: () => props.onDone(),
  });
  const deleteMutation = useDeleteChallengeFixMutation({
    onSuccess: () => props.onDone(),
  });

  function save() {
    let testerCommand: string[] | null = null;
    try {
      const parsed = JSON.parse(commandText() || "[]");
      if (!Array.isArray(parsed) || parsed.some((item) => typeof item !== "string")) {
        throw new Error("tester command must be a string array");
      }
      testerCommand = parsed.length > 0 ? parsed : null;
    } catch (err) {
      addToast({
        level: "error",
        description: `${t("challenge.fix.form.testerCommand.invalid")}: ${err}`,
        duration: 5000,
      });
      return;
    }
    updateMutation.mutate({
      game_id: props.gameId,
      challenge_id: props.challengeId,
      config: {
        ...config(),
        tester_command: testerCommand,
      },
    });
  }

  return (
    <Card contentClass="p-3 flex flex-col space-y-3">
      <header class="min-h-10 flex flex-row flex-wrap gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--wrench-20-regular] w-5 h-5" />
        <span class="font-bold flex-1">{t("challenge.fix.admin")}</span>
        <Checkbox
          checked={config().enabled}
          onChange={() => setConfig((current) => ({ ...current, enabled: !current.enabled }))}
        >
          <span>{t("challenge.fix.form.enabled")}</span>
        </Checkbox>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          type="number"
          title={t("challenge.fix.form.maxAttempts")}
          value={config().max_attempts}
          onInput={(e) => setConfig((current) => ({ ...current, max_attempts: Number(e.currentTarget.value) || 1 }))}
        />
        <Input
          title={t("challenge.fix.form.fixScript")}
          value={config().fix_script}
          onInput={(e) => setConfig((current) => ({ ...current, fix_script: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.fix.form.uploadPath")}
          value={config().upload_path}
          onInput={(e) => setConfig((current) => ({ ...current, upload_path: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.fix.form.targetContainer")}
          value={config().target_container ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, target_container: e.currentTarget.value || null }))}
        />
        <Input
          type="number"
          title={t("challenge.fix.form.targetPort")}
          value={config().target_port ?? ""}
          onInput={(e) =>
            setConfig((current) => ({
              ...current,
              target_port: Number(e.currentTarget.value) || null,
            }))
          }
        />
        <Input
          type="number"
          title={t("challenge.fix.form.timeout")}
          value={config().timeout_secs}
          onInput={(e) => setConfig((current) => ({ ...current, timeout_secs: Number(e.currentTarget.value) || 120 }))}
        />
        <Input
          title={t("challenge.fix.form.resultEnv")}
          value={config().result_env}
          onInput={(e) => setConfig((current) => ({ ...current, result_env: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.fix.form.successValue")}
          value={config().success_value}
          onInput={(e) => setConfig((current) => ({ ...current, success_value: e.currentTarget.value }))}
        />
        <Input
          title={t("challenge.fix.form.pullSecret")}
          value={config().pull_secret ?? ""}
          onInput={(e) => setConfig((current) => ({ ...current, pull_secret: e.currentTarget.value || null }))}
        />
      </div>
      <header class="min-h-10 flex flex-row gap-2 items-center border-b border-b-layer-content/10 pb-2">
        <span class="shrink-0 icon-[fluent--beaker-20-regular] w-5 h-5" />
        <span class="font-bold">{t("challenge.fix.tester")}</span>
      </header>
      <div class="grid grid-cols-fit-xs gap-2">
        <Input
          title={t("challenge.instance.image.form.containerName.label")}
          value={config().tester?.name ?? ""}
          onInput={(e) => updateTester({ name: e.currentTarget.value })}
        />
        <Input
          title={t("challenge.instance.image.form.tag.label")}
          value={config().tester?.tag ?? ""}
          onInput={(e) => updateTester({ tag: e.currentTarget.value })}
        />
        <Input
          type="number"
          title={t("challenge.instance.image.form.service.cpu.label")}
          value={config().tester?.cpu ?? 0.5}
          onInput={(e) => updateTester({ cpu: Number(e.currentTarget.value) || 0.5 })}
        />
        <Input
          title={t("challenge.instance.image.form.service.mem.label")}
          value={config().tester?.mem ?? "128Mi"}
          onInput={(e) => updateTester({ mem: e.currentTarget.value })}
        />
      </div>
      <label class="flex flex-col space-y-1">
        <span class="label">{t("challenge.fix.form.testerCommand.label")}</span>
        <textarea
          class="input min-h-28 font-mono text-sm"
          value={commandText()}
          onInput={(e) => setCommandText(e.currentTarget.value)}
        />
      </label>
      <div class="flex flex-row justify-end gap-2">
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

export default function Fix(props: ChallengeWidgetProps) {
  const game = useGame({ id: () => props.gameId });
  const fix = useChallengeFix({ game_id: () => props.gameId, challenge_id: () => props.challengeId });
  const [file, setFile] = createSignal<File | null>(null);
  const [pendingSubmissionId, setPendingSubmissionId] = createSignal<number | null>(null);
  const [pendingText, setPendingText] = createSignal("");
  let inputRef: HTMLInputElement;

  const submitMutation = useSubmitFixMutation({
    onSuccess: async (submission) => {
      setPendingSubmissionId(submission.id);
      setPendingText(t("challenge.submission.status.pending.title"));
      let iter = 0;
      while (iter < 180) {
        const status = await checkSubmissionStatus(props.gameId, props.challengeId, submission.id);
        if (status.solved !== null) {
          setPendingText(
            `${status.solved ? t("challenge.submission.status.solved.title") : t("challenge.submission.status.failed.title")}: ${status.result ?? ""}`
          );
          fix.refetch();
          inflyClient.invalidateQueries({ queryKey: ["game", props.gameId, "challenge"] });
          inflyClient.invalidateQueries({ queryKey: ["game", props.gameId, "selfSolves"] });
          if (isGameInProgress(game.data) && !isAdminOfGame(game.data)) {
            inflyClient.invalidateQueries({ queryKey: ["game", props.gameId, "team", "self"] });
          }
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 1000));
        iter += 1;
      }
    },
  });

  return (
    <div class="flex-1 flex flex-col space-y-3 p-3 lg:p-6">
      <Show when={fix.isLoading}>
        <LoadingTips />
      </Show>
      <Show when={fix.data?.config?.enabled} fallback={<Card contentClass="p-3">{t("challenge.fix.disabled")}</Card>}>
        <Card contentClass="p-3 flex flex-col gap-3">
          <header class="min-h-10 flex flex-row items-center gap-2 border-b border-b-layer-content/10 pb-2">
            <span class="shrink-0 icon-[fluent--wrench-20-regular] w-5 h-5" />
            <span class="font-bold flex-1">{t("challenge.fix.title")}</span>
            <span class="opacity-70">
              {t("challenge.fix.attempts", {
                used: fix.data?.attempts_used ?? 0,
                total: fix.data?.config?.max_attempts ?? 0,
              })}
            </span>
          </header>
          <Show when={fix.data?.solved}>
            <Card level="success" contentClass="p-2 flex flex-row gap-2 items-center">
              <span class="shrink-0 icon-[fluent--checkmark-circle-20-filled] w-5 h-5" />
              <span>{t("challenge.fix.solved")}</span>
            </Card>
          </Show>
          <div class="flex flex-row flex-wrap gap-2 items-center">
            <Button onClick={() => inputRef!.click()} disabled={submitMutation.isPending || fix.data?.solved}>
              <span class="shrink-0 icon-[fluent--folder-open-20-regular] w-5 h-5" />
              <span>{file() ? file()!.name : t("general.actions.select.title")}</span>
              <Show when={file()}>
                <span class="opacity-60">{humanFileSize(file()!.size, true)}</span>
              </Show>
            </Button>
            <Button
              level="primary"
              disabled={!file() || submitMutation.isPending || fix.data?.solved}
              loading={submitMutation.isPending}
              onClick={() => {
                const selected = file();
                if (!selected) return;
                submitMutation.mutate({ game_id: props.gameId, challenge_id: props.challengeId, file: selected });
              }}
            >
              {t("challenge.fix.submit")}
            </Button>
            <input
              hidden
              class="hidden"
              ref={inputRef!}
              type="file"
              onChange={(e) => setFile(e.currentTarget.files?.[0] ?? null)}
            />
          </div>
          <Show when={pendingSubmissionId()}>
            <Card contentClass="p-2 flex flex-row gap-2 items-center">
              <span class="shrink-0 icon-[fluent--flash-play-20-regular] w-5 h-5" />
              <span>#{pendingSubmissionId()}</span>
              <span class="flex-1">{pendingText()}</span>
            </Card>
          </Show>
        </Card>
      </Show>
      <Show when={isAdminOfGame(game.data)}>
        <AdminFixPanel
          gameId={props.gameId}
          challengeId={props.challengeId}
          config={fix.data?.config ?? null}
          onDone={() => fix.refetch()}
        />
      </Show>
    </div>
  );
}
