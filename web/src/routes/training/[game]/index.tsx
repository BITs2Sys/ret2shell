import type { Challenge as ChallengeModel } from "@/lib/models/challenge";
import { gameStore, refreshChallenges, setGameStore } from "@/lib/storage/game";
import { t } from "@/lib/storage/theme";
import LoadingTips from "@/lib/widgets/loading-tips";
import Challenge from "@blocks/challenge";
import { useSearchParams } from "@solidjs/router";
import { Match, Switch, createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import Intro from "../_blocks/intro";

import Form, { type ChallengeForm } from "@/lib/blocks/challenge/form";
import { DateTime } from "luxon";
import { createChallenge, getChallenge } from "@/lib/api/game";
import type { HTTPError } from "ky";
import { addToast } from "@/lib/storage/toast";
import Tabs from "@/lib/blocks/challenge/tabs";

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
        refreshChallenges();
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
  const [selectedChallenge, setSelectedChallenge] = createSignal(null as null | ChallengeModel);
  createEffect(() => {
    if (selectedChallengeId() && gameStore.current) {
      untrack(() => {
        setLoadingChallenge(true);
        getChallenge(gameStore.current!.id, selectedChallengeId()!)
          .then((resp) => {
            setSelectedChallenge(resp);
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
      setSelectedChallenge(null);
    }
  });
  onCleanup(() => {
    setGameStore({ current: null });
  });

  return (
    <div class="flex-1 flex flex-col w-0">
      <Tabs baseUrl={`/training/${gameStore.current?.id}`} current={selectedChallenge()} loading={loadingChallenge()} />
      <Switch fallback={<Intro />}>
        <Match when={loadingChallenge()}>
          <div class="flex-1 flex flex-row space-x-2 items-center justify-center">
            <LoadingTips />
          </div>
        </Match>
        <Match when={inCreate()}>
          <Form onDone={onCreateChallenge} loading={creating()} />
        </Match>
        <Match when={selectedChallenge()}>
          <Challenge challenge={selectedChallenge()!} />
        </Match>
      </Switch>
    </div>
  );
}
