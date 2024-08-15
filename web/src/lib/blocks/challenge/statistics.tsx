import { getChallengeCommitHistory } from "@api/game";
import type { Challenge, CommitHistory } from "@models/challenge";
import { challengeStore } from "@storage/challenge";
import { gameStore } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Divider from "@widgets/divider";
import LoadingTips from "@widgets/loading-tips";
import type { HTTPError } from "ky";
import { DateTime } from "luxon";
import { For, Match, Show, Switch, createEffect, createSignal, untrack } from "solid-js";

function StatisticsPanel() {
  return <></>;
}
function HintsPanel() {
  return <></>;
}

function HistoryPanel() {
  const [history, setHistory] = createSignal([] as CommitHistory[]);
  const [loading, setLoading] = createSignal(false);
  createEffect(() => {
    if (challengeStore.current) {
      untrack(() => {
        setLoading(true);
        getChallengeCommitHistory(gameStore.current!.id, challengeStore.current!.id)
          .then(setHistory)
          .catch((err: HTTPError) => {
            err.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.challenge.fetchHistoryFailed")}: ${text}`,
                duration: 5000,
              });
            });
          })
          .finally(() => {
            setLoading(false);
          });
      });
    }
  });
  return (
    <>
      <Show when={loading()}>
        <div class="w-full flex flex-row space-x-2 p-2 items-center border-b border-b-layer-content/10 overflow-hidden">
          <LoadingTips />
        </div>
      </Show>
      <For each={history()}>
        {(item) => (
          <div class="w-full flex flex-row space-x-2 p-2 items-center border-b border-b-layer-content/10 overflow-hidden">
            <span class="icon-[fluent--branch-request-20-regular] w-5 h-5 text-primary" />
            <span class="flex-1 truncate w-0">{item.subject}</span>
            <span class="font-bold">{item.author.name}</span>
            <span class="font-bold text-primary opacity-40">{item.abbreviated_commit}</span>
            <span class="opacity-40">{DateTime.fromSeconds(item.author.date).toFormat("yyyy-MM-dd HH:mm:ss")}</span>
          </div>
        )}
      </For>
    </>
  );
}

export default function (_props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
}) {
  const [tab, setTab] = createSignal("statistics" as "statistics" | "history" | "hints");
  return (
    <div class="flex flex-row min-h-full">
      <ul class="w-1/5 min-w-48 flex flex-col space-y-2 p-3 lg:p-6 sticky top-0 self-start">
        <li class="w-full">
          <Button ghost={tab() !== "statistics"} class="h-auto w-full" onClick={() => setTab("statistics")}>
            <div class="flex flex-col py-2 items-start w-full">
              <span>{t("game.challenge.statistics")}</span>
              <span class="font-normal opacity-60">{t("game.challenge.statisticsTips")}</span>
            </div>
          </Button>
        </li>
        <li class="w-full">
          <Button ghost={tab() !== "history"} class="h-auto w-full" onClick={() => setTab("history")}>
            <div class="flex flex-col py-2 items-start w-full">
              <span>{t("game.challenge.commitHistory")}</span>
              <span class="font-normal opacity-60">{t("game.challenge.commitHistoryTips")}</span>
            </div>
          </Button>
        </li>
        <li class="w-full">
          <Button ghost={tab() !== "hints"} class="h-auto w-full" onClick={() => setTab("hints")}>
            <div class="flex flex-col py-2 items-start w-full">
              <span>{t("game.challenge.hintHistory")}</span>
              <span class="font-normal opacity-60">{t("game.challenge.hintHistoryTips")}</span>
            </div>
          </Button>
        </li>
      </ul>
      <Divider direction="vertical" />
      <div class="flex-1 w-0 flex flex-col space-y-2 p-3 lg:p-6">
        <Switch>
          <Match when={tab() === "statistics"}>
            <StatisticsPanel />
          </Match>
          <Match when={tab() === "history"}>
            <HistoryPanel />
          </Match>
          <Match when={tab() === "hints"}>
            <HintsPanel />
          </Match>
        </Switch>
      </div>
    </div>
  );
}
