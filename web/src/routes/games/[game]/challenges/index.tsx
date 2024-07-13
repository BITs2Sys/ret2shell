import Form, { type ChallengeForm } from "@/lib/blocks/challenge/form";
import LoadingTips from "@/lib/widgets/loading-tips";
import Challenge from "@blocks/challenge";
import ChallengeList from "@blocks/challenge/list";
import SidebarLayout from "@blocks/sidebar-layout";
import type { Challenge as ChallengeModel } from "@models/challenge";
import { useNavigate, useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { gameStore, refreshChallenges } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Link from "@widgets/link";

import { Match, Switch, createEffect, createMemo, createSignal, untrack } from "solid-js";
import Notifications from "./_blocks/notifications";
import Team from "./_blocks/team";
import Welcome from "./_blocks/welcome";
import { DateTime } from "luxon";
import { createChallenge, getChallenge } from "@/lib/api/game";
import type { HTTPError } from "ky";
import { addToast } from "@/lib/storage/toast";

import Tabs from "@/lib/blocks/challenge/tabs";

export default function () {
  const navigate = useNavigate();
  if (accountStore.token === null) {
    navigate(`/account/login?redirect=/games/${gameStore.current ? gameStore.current.id : ""}`);
    return null;
  }
  const [loadingChallenge, setLoadingChallenge] = createSignal(false);
  const [searchParams, setSearchParams] = useSearchParams();
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
              setSearchParams({ challenge: null, create: null });
            });
          })
          .finally(() => {
            setLoadingChallenge(false);
          });
      });
    } else {
      setSelectedChallenge(null);
    }
  });

  const inCreate = createMemo(() => searchParams.create === "true");
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
        initial: result.initial,
        minimum: result.minimum,
        decay: result.decay,
      },
      score: result.initial,
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
  // TODO: fetchSelfTeam and redirect
  return (
    <>
      <Title title={`${t("game.challenge.title")} - ${gameStore.current?.name || "CTF"}`} />
      <SidebarLayout
        leftBar={() => (
          <>
            <div class="border-b border-b-layer-content/10 px-2 h-16 flex items-center justify-center">
              <Link class="w-full" ghost justify="start" href={`/games/${gameStore.current?.id}/challenges`}>
                <span class="icon-[fluent--flag-20-filled] w-5 h-5 text-primary" />
                <span>{t("game.challenge.list")}</span>
              </Link>
            </div>
            <ChallengeList showScore inGame />
          </>
        )}
        rightBar={() => (
          <div class="flex flex-col">
            <Team />
            <Notifications />
          </div>
        )}
      >
        <div class="flex-1 flex flex-col w-0">
          <Tabs
            baseUrl={`/games/${gameStore.current?.id}/challenges`}
            current={selectedChallenge()}
            loading={loadingChallenge()}
          />
          <Switch fallback={<Welcome />}>
            <Match when={loadingChallenge()}>
              <div class="flex-1 flex flex-row space-x-2 items-center justify-center">
                <LoadingTips />
              </div>
            </Match>
            <Match when={inCreate()}>
              <Form onDone={onCreateChallenge} loading={creating()} inGame />
            </Match>
            <Match when={selectedChallenge()}>
              <Challenge inGame challenge={selectedChallenge()!} />
            </Match>
          </Switch>
        </div>
      </SidebarLayout>
    </>
  );
}
