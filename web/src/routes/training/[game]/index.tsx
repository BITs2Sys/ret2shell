import Challenge from "@blocks/challenge";
import type { Challenge as ChallengeModel } from "@models/challenge";
import { useSearchParams } from "@solidjs/router";
import { gameStore, setGameStore } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import LoadingTips from "@widgets/loading-tips";
import { Match, Switch, createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import Intro from "../_blocks/intro";

import { createChallenge, getChallenge, updateGame } from "@api/game";
import Form, { type ChallengeForm } from "@blocks/challenge/form";
import Tabs from "@blocks/challenge/tabs";
import AdministratorsManagement from "@blocks/game/administrators";
import GameEdit, { type GameForm } from "@blocks/game/form";
import GameStatistics from "@blocks/game/statistics";
import { challengeStore, refreshChallengeAssets, refreshChallenges, setChallengeStore } from "@storage/challenge";
import { addToast } from "@storage/toast";
import type { HTTPError } from "ky";
import { DateTime } from "luxon";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";

export default function () {
  const [searchParams, setSearchParams] = useSearchParams();
  const inCreate = createMemo(() => searchParams.create === "true");
  const [loadingChallenge, setLoadingChallenge] = createSignal(false);
  const [creating, setCreating] = createSignal(false);

  function onCreateChallenge(result: ChallengeForm) {
    setCreating(true);
    const tags = result.tag.split("/").map((t) => {
      return { name: t, primary: false };
    });
    tags[0].primary = true;

    const challenge = {
      id: 0,
      name: result.name,
      updated_at: DateTime.now(),
      hidden: true,
      content: result.content,
      game_id: gameStore.current?.id,
      tag: tags,
      score_rule: {
        initial: 1,
        minimum: 1,
        decay: 1,
      },
      score: 1,
      bucket: null,
    } as ChallengeModel;
    createChallenge(gameStore.current!.id, challenge)
      .then((result) => {
        setSearchParams({
          create: null,
          challenge: result.id,
        });
        void refreshChallenges();
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.createFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setCreating(false);
      });
  }
  const selectedChallengeId = createMemo(() => Number.parseInt(searchParams.challenge || "NaN") || null);
  const inEdit = createMemo(() => searchParams.edit === "true");
  const inStatistics = createMemo(() => searchParams.statistics === "true");
  createEffect(() => {
    if (selectedChallengeId() && gameStore.current) {
      untrack(() => {
        setLoadingChallenge(true);
        getChallenge(gameStore.current!.id, selectedChallengeId()!)
          .then((resp) => {
            setChallengeStore({ current: resp });
            refreshChallengeAssets();
          })
          .catch((e: HTTPError) => {
            e.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.challenge.fetchChallengeFailed")}: ${text}`,
                duration: 5000,
              });
            });
            setSearchParams({ challenge: null, create: null });
          })
          .finally(() => {
            setLoadingChallenge(false);
          });
      });
    } else {
      setChallengeStore({ current: null });
    }
  });
  onCleanup(() => {
    setGameStore({ current: null });
  });

  const [editing, setEditing] = createSignal(false);

  function onEditGame(result: GameForm) {
    setEditing(true);
    updateGame(gameStore.current!.id, {
      ...gameStore.current!,
      ...result,
      start_at: DateTime.fromFormat("2002-05-05 10:00", "yyyy-MM-dd HH:mm"),
      end_at: DateTime.fromFormat("2077-01-01 10:00", "yyyy-MM-dd HH:mm"),
      archive_at: DateTime.fromFormat("2077-01-01 10:00", "yyyy-MM-dd HH:mm"),
      register_at: DateTime.fromFormat("2002-05-05 10:00", "yyyy-MM-dd HH:mm"),
    })
      .then((game) => {
        setGameStore({ current: game });
        addToast({
          level: "success",
          description: t("form.saveSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.saveFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setEditing(false);
      });
  }

  return (
    <div class="flex-1 flex flex-col w-0">
      <Tabs baseUrl={`/training/${gameStore.current?.id}`} loading={loadingChallenge()} />
      <Switch fallback={<Intro />}>
        <Match when={inEdit()}>
          <div class="flex-1 w-full relative">
            <div class="absolute top-0 left-0 w-full h-full overflow-hidden">
              <OverlayScrollbarsComponent
                options={{
                  scrollbars: {
                    theme: `os-theme-${fullTheme()}`,
                    autoHide: "scroll",
                  },
                }}
                class="relative w-full h-full print:h-auto print:overflow-auto"
                defer
              >
                <div class="w-full flex flex-col p-3 lg:p-6 items-center">
                  <GameEdit onDone={onEditGame} loading={editing()} editSource={gameStore.current || undefined} />
                  <div class="h-16" />
                  <div class="w-full max-w-5xl flex flex-col space-y-2 relative">
                    <AdministratorsManagement />
                  </div>
                </div>
              </OverlayScrollbarsComponent>
            </div>
          </div>
        </Match>
        <Match when={inStatistics()}>
          <div class="flex-1 w-full relative">
            <div class="absolute top-0 left-0 w-full h-full overflow-hidden">
              <OverlayScrollbarsComponent
                options={{
                  scrollbars: {
                    theme: `os-theme-${fullTheme()}`,
                    autoHide: "scroll",
                  },
                }}
                class="relative w-full h-full print:h-auto print:overflow-auto"
                defer
              >
                <div class="w-full flex flex-col p-3 lg:p-6 items-center">
                  <GameStatistics />
                </div>
              </OverlayScrollbarsComponent>
            </div>
          </div>
        </Match>
        <Match when={loadingChallenge()}>
          <div class="flex-1 flex flex-row space-x-2 items-center justify-center">
            <LoadingTips />
          </div>
        </Match>
        <Match when={inCreate()}>
          <Form onDone={onCreateChallenge} loading={creating()} />
        </Match>
        <Match when={challengeStore.current}>
          <Challenge onStateChange={refreshChallenges} />
        </Match>
      </Switch>
    </div>
  );
}
