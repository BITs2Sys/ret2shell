import { api_root } from "@api";
import { getCalmdownStatus } from "@api/cluster";
import { delayGameSelfEnv, startChallengeEnv, stopGameSelfEnv } from "@api/game";
import Spin from "@assets/animates/spin";
import { getWsrxLink, wsrx } from "@lib/wsrx";
import { accountStore } from "@storage/account";
import { challengeStore, refreshStatus } from "@storage/challenge";
import { fullTheme, t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import Button from "@widgets/button";
import ClipboardBtn from "@widgets/clipboard-btn";
import Divider from "@widgets/divider";
import Tag from "@widgets/tag";
import TimeProgress from "@widgets/time-progress";
import Timer from "@widgets/timer";
import type { HTTPError } from "ky";
import type { DateTime } from "luxon";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { passiveSupport } from "passive-events-support/src/utils";
import { For, Match, Show, Switch, createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import DownloadButton from "../download-button";

passiveSupport({
  events: ["mousewheel", "wheel"],
  listeners: [
    {
      element: "challenge-header",
      event: "wheel",
    },
  ],
});

export default function (props: { inGame?: boolean }) {
  const instance = createMemo(() => {
    if (challengeStore.current && challengeStore.env) {
      return wsrx.instances().find((s) => s.challenge_id === challengeStore.current!.id) ?? null;
    }
    return null;
  });
  const [calmdownStart, setCalmdownStart] = createSignal<DateTime | null>(null);
  async function refreshCalmdown() {
    try {
      const result = await getCalmdownStatus();
      setCalmdownStart(result);
    } catch {
      setCalmdownStart(null);
      return;
    }
  }
  createEffect(() => {
    if (challengeStore.current && challengeStore.env) {
      untrack(refreshCalmdown);
    }
  });
  createEffect(() => {
    if (challengeStore.current) {
      untrack(refreshStatus);
    }
  });
  let instanceStateIter = 0;
  function maintainInstances() {
    wsrx.refreshInstances().then(() => {
      wsrx.deleteOutdatedTraffic();
      wsrx.openAllTraffic().then(() => {
        wsrx.refreshTraffic();
      });
    });
  }
  function maintainInstancesWorker() {
    if (instance()?.state === "Pending" || instanceStateIter === 0) {
      maintainInstances();
    }
    instanceStateIter++;
    instanceStateIter = instanceStateIter % 20;
    return maintainInstancesWorker;
  }
  const timer = setInterval(maintainInstancesWorker(), 1000);
  onCleanup(() => {
    clearInterval(timer);
  });
  const userExplicitInstance = createMemo(() => {
    if (challengeStore.current && challengeStore.env) {
      return wsrx.instances().find((s) => s.user_id === accountStore.id) ?? null;
    }
    return null;
  });

  const [starting, setStarting] = createSignal(false);
  function handleStartChallengeEnv() {
    setStarting(true);
    startChallengeEnv(challengeStore.current!.game_id, challengeStore.current!.id)
      .then(() => {
        setTimeout(() => {
          maintainInstances();
        }, 500);
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.startEnvFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setStarting(false);
      });
  }

  const [delaying, setDelaying] = createSignal(false);
  function handleDelaySelfEnv() {
    setDelaying(true);
    setTimeout(() => {
      delayGameSelfEnv(challengeStore.current!.game_id)
        .then(() => {
          setTimeout(() => {
            maintainInstances();
          }, 500);
        })
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("game.challenge.delayEnvFailed")}: ${text}`,
              duration: 5000,
            });
          });
        })
        .finally(() => {
          setDelaying(false);
        });
    }, 500);
  }
  const [stopping, setStopping] = createSignal(false);
  function handleStopSelfEnv() {
    setStopping(true);
    setTimeout(() => {
      stopGameSelfEnv(challengeStore.current!.game_id)
        .then(() => {
          addToast({
            level: "success",
            description: t("game.challenge.stopEnvSuccess")!,
            duration: 5000,
          });
          setTimeout(() => {
            maintainInstances();
            refreshCalmdown();
          }, 500);
        })
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("game.challenge.stopEnvFailed")}: ${text}`,
              duration: 5000,
            });
          });
        })
        .finally(() => {
          setStopping(false);
        });
    }, 500);
  }
  return (
    <div class="w-full h-full overflow-hidden flex flex-col">
      <OverlayScrollbarsComponent
        options={{
          scrollbars: {
            theme: `os-theme-${fullTheme()}`,
            autoHide: "scroll",
          },
        }}
        class="relative w-full flex-1 print:h-auto print:overflow-auto"
        defer
      >
        <div class="flex flex-col items-center p-3 lg:p-6">
          <div class="flex flex-col w-full max-w-5xl">
            <header class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-6 font-bold">
              <span class="flex flex-row space-x-2 items-center flex-1 overflow-hidden">
                <span class="icon-[fluent--info-20-regular] w-5 h-5" />
                <span class="flex-1 truncate">{challengeStore.current?.name}</span>
              </span>
              <Show when={props.inGame}>
                <span
                  class={`font-bold flex flex-row space-x-2 items-center ${challengeStore.status?.solved ? "text-success" : "text-warning"}`.trim()}
                >
                  <span
                    class={
                      challengeStore.status?.solved
                        ? "icon-[fluent--checkmark-circle-20-regular] w-5 h-5"
                        : "icon-[fluent--flag-20-regular] w-5 h-5"
                    }
                  />
                  <span>{challengeStore.current?.score} pts</span>
                </span>
              </Show>
              <span class="font-bold flex flex-row space-x-2 items-center">
                <span class="icon-[fluent--data-bar-vertical-24-regular] w-5 h-5" />
                <span>
                  {challengeStore.status?.solves ?? 0} solve
                  {challengeStore.status?.solves && challengeStore.status.solves > 1 ? "s" : ""}
                </span>
              </span>
            </header>
            <Show when={challengeStore.files.length > 0}>
              <section class="inline-flex flex-row items-center flex-wrap min-h-12 border-b border-b-layer-content/15">
                <h3 class="font-bold flex space-x-2 items-center">
                  <span class="icon-[fluent--folder-zip-20-regular] w-5 h-5" />
                  <span>{t("game.challenge.files")}:</span>
                </h3>
                <div class="w-4" />
                <For each={challengeStore.files}>
                  {(file) => (
                    <DownloadButton
                      class="m-1"
                      size="sm"
                      file={file.file}
                      withFileName
                      icon="icon-[fluent--arrow-download-20-regular]"
                      url={`${api_root}/game/${challengeStore.current!.game_id}/challenge/${challengeStore.current!.id}/file`}
                      searchParams={{ file: file.file, folder: file.folder }}
                    />
                  )}
                </For>
              </section>
            </Show>
            <Show when={challengeStore.env}>
              <section class="h-12 border-b border-b-layer-content/15 flex flex-row items-center relative">
                <Show when={instance()}>
                  <TimeProgress
                    class="absolute bottom-0 left-0 right-0"
                    startAt={instance()!.created_at}
                    endAt={instance()!.created_at.plus({ hours: instance()!.renew_count + 1 })}
                  />
                </Show>
                <h3 class="font-bold flex space-x-2 items-center flex-1">
                  <span class={`icon-[fluent--play-20-regular] w-5 h-5 ${instance() ? "text-success" : ""}`.trim()} />
                  <Switch fallback={<span class="opacity-80 flex-1 truncate">{t("game.challenge.envNotStart")}</span>}>
                    <Match when={instance()?.state === "Running"}>
                      <span class="flex-1 truncate">
                        {t("game.challenge.envIsRunning")}: {instance()?.wsrx}
                      </span>
                    </Match>
                    <Match when={instance()?.state === "Pending"}>
                      <Spin width={20} height={20} />
                      <span class="flex-1 truncate">{t("game.challenge.envIsPending")}</span>
                    </Match>
                    <Match when={instance()}>
                      <Spin width={20} height={20} />
                      <span class="flex-1 truncate text-error">{t("game.challenge.envHasError")}</span>
                    </Match>
                    <Match when={userExplicitInstance()}>
                      <span class="text-warning flex-1 truncate">
                        {t("game.challenge.otherChallengeEnvIsRunning")}: {userExplicitInstance()?.challenge_name}
                      </span>
                    </Match>
                  </Switch>
                </h3>
                <div class="flex flex-row space-x-2 items-center">
                  <Switch
                    fallback={
                      <Button
                        ghost
                        size="sm"
                        onClick={handleStartChallengeEnv}
                        loading={starting()}
                        disabled={starting() || calmdownStart() !== null}
                      >
                        <Show
                          when={calmdownStart()}
                          fallback={
                            <>
                              <span class="icon-[fluent--play-20-regular] w-5 h-5 text-success" />
                              <span>{t("game.challenge.startEnv")}</span>
                            </>
                          }
                        >
                          <span class="icon-[fluent--history-20-regular] w-5 h-5" />
                          <span class="opacity-60">{t("game.challenge.calmDownBeforeStartEnv")}</span>
                          <Timer
                            end={calmdownStart()!.plus({ minutes: 1 })}
                            onTimeout={() => {
                              setTimeout(() => {
                                refreshCalmdown();
                              }, 500);
                            }}
                          />
                        </Show>
                      </Button>
                    }
                  >
                    <Match when={instance()}>
                      {/* <For each={challengeStore.env?.images}>
                        {(image) => (
                          <Show when={image.port}>
                            <ClipboardBtn
                              size="sm"
                              title={image.description!}
                              value={getWsrxLink(instance()!.wsrx, image.port!)}
                            >
                              <Switch>
                                <Match when={image.service_type === "http"}>
                                  <span class="text-info">HTTP</span>
                                </Match>
                                <Match when={image.service_type === "tcp"}>
                                  <span class="text-warning">TCP</span>
                                </Match>
                              </Switch>
                              <span>{image.port}</span>
                            </ClipboardBtn>
                          </Show>
                        )}
                      </For> */}
                      <Timer
                        end={instance()!.created_at.plus({ hours: instance()!.renew_count + 1 })}
                        onTimeout={() => {
                          setTimeout(() => {
                            maintainInstances();
                          }, 500);
                        }}
                        hasHours
                      />
                      <Divider class="h-8" direction="horizontal" />
                      <Button
                        ghost
                        size="sm"
                        title={t("game.challenge.delayEnv")}
                        square
                        onClick={handleDelaySelfEnv}
                        loading={delaying()}
                        disabled={delaying()}
                      >
                        <Show when={!delaying()}>
                          <span class="icon-[fluent--clock-alarm-20-regular] w-5 h-5 text-primary" />
                        </Show>
                      </Button>
                      <Button
                        ghost
                        size="sm"
                        title={t("game.challenge.stopEnv")}
                        square
                        onClick={handleStopSelfEnv}
                        loading={stopping()}
                        disabled={stopping()}
                      >
                        <Show when={!stopping()}>
                          <span class="icon-[fluent--record-stop-20-regular] w-5 h-5 text-error" />
                        </Show>
                      </Button>
                    </Match>
                    <Match when={userExplicitInstance()}>
                      <Button
                        ghost
                        size="sm"
                        square
                        title={t("game.challenge.stopEnv")}
                        onClick={handleStopSelfEnv}
                        loading={stopping()}
                        disabled={stopping()}
                      >
                        <Show when={!stopping()}>
                          <span class="icon-[fluent--record-stop-20-regular] w-5 h-5 text-error" />
                        </Show>
                      </Button>
                    </Match>
                  </Switch>
                </div>
              </section>
              <Show when={instance()}>
                <For each={challengeStore.env?.images}>
                  {(image) => (
                    <Show when={image.port}>
                      <section class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 relative">
                        <span class="icon-[fluent--cube-20-regular] w-5 h-5 text-info" />
                        <span class="flex-1 text-start">{image.name}.service</span>
                        <ClipboardBtn
                          size="sm"
                          title={image.description!}
                          value={getWsrxLink(instance()!.wsrx, image.port!)}
                          label="WSRX"
                        />
                        <Show when={wsrx.getTrafficLocal(instance()!, image.port!)}>
                          <ClipboardBtn
                            size="sm"
                            title={image.description!}
                            value={wsrx.getTrafficLocal(instance()!, image.port!)?.local}
                            label={wsrx.getTrafficLocal(instance()!, image.port!)?.local}
                          />
                        </Show>
                      </section>
                    </Show>
                  )}
                </For>
              </Show>
            </Show>
            <Show when={challengeStore.current}>
              <div class="flex flex-row-reverse flex-wrap py-2">
                <For each={challengeStore.current!.tag}>
                  {(tag) => (
                    <Tag level={tag.primary ? "success" : "info"} class="m-1">
                      <span>{tag.name}</span>
                    </Tag>
                  )}
                </For>
              </div>
            </Show>
            <Article content={challengeStore.current?.content ?? ""} extra />
          </div>
        </div>
      </OverlayScrollbarsComponent>
    </div>
  );
}
