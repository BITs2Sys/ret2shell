import { api_root } from "@api";
import { wsrx } from "@lib/wsrx";
import { accountStore } from "@storage/account";
import { challengeStore } from "@storage/challenge";
import { fullTheme, t } from "@storage/theme";
import Article from "@widgets/article";
import Button from "@widgets/button";
import Divider from "@widgets/divider";
import Tag from "@widgets/tag";
import TimeProgress from "@widgets/time-progress";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { passiveSupport } from "passive-events-support/src/utils";
import { For, Match, Show, Switch, createMemo } from "solid-js";
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

export default function (props: { solved?: boolean; solves?: number; inGame?: boolean }) {
  const instance = createMemo(() => {
    if (challengeStore.current && challengeStore.env) {
      return wsrx.instances().find((s) => s.challenge_id === challengeStore.current!.id) ?? null;
    }
    return null;
  });
  const localAddr = createMemo(() => {
    if (instance()) {
      return wsrx.traffic().find((t) => t.wsrx === instance()!.wsrx)?.local_addr ?? null;
    }
    return null;
  });
  const userExplicitInstance = createMemo(() => {
    if (challengeStore.current && challengeStore.env) {
      return wsrx.instances().find((s) => s.user_id === accountStore.id) ?? null;
    }
    return null;
  });
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
                  class={`font-bold flex flex-row space-x-2 items-center ${props.solved ? "text-success" : "text-warning"}`.trim()}
                >
                  <span
                    class={
                      props.solved
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
                  {props.solves ?? 0} solve{props.solves && props.solves > 1 ? "s" : ""}
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
              <section class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-6 relative">
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
                    <Match when={instance()}>
                      <span class="flex-1 truncate">
                        {t("game.challenge.envIsRunning")}: {localAddr() ?? instance()?.wsrx}
                      </span>
                    </Match>
                    <Match when={userExplicitInstance()}>
                      <span class="text-warning flex-1 truncate">
                        {t("game.challenge.otherChallengeEnvIsRunning")}: {userExplicitInstance()?.challenge_name}
                      </span>
                    </Match>
                  </Switch>
                </h3>
                <div class="flex flex-row space-x-2">
                  <Switch
                    fallback={
                      <Button ghost size="sm">
                        <span class="icon-[fluent--play-20-regular] w-5 h-5 text-success" />
                        <span>{t("game.challenge.startEnv")}</span>
                        {/* <span class="icon-[fluent--history-20-regular] w-5 h-5" />
                        <span class="opacity-60">{t("game.challenge.calmDownBeforeStartEnv")}</span>
                        <span>00:48</span> */}
                      </Button>
                    }
                  >
                    <Match when={instance()}>
                      <For each={challengeStore.env?.images}>
                        {(image) => (
                          <Show when={image.port}>
                            <Button ghost size="sm" title={image.description}>
                              <Switch>
                                <Match when={image.service_type === "http"}>
                                  <span class="text-info">HTTP</span>
                                </Match>
                                <Match when={image.service_type === "tcp"}>
                                  <span class="text-warning">TCP</span>
                                </Match>
                              </Switch>
                              <span>{image.port}</span>
                            </Button>
                          </Show>
                        )}
                      </For>
                      <Divider class="h-8" direction="horizontal" />
                      <Button ghost size="sm" square>
                        <span class="icon-[fluent--clock-alarm-20-regular] w-5 h-5 text-primary" />
                      </Button>
                      <Button ghost size="sm" square>
                        <span class="icon-[fluent--record-stop-20-regular] w-5 h-5 text-primary" />
                      </Button>
                    </Match>
                    <Match when={userExplicitInstance()}>
                      <Button ghost size="sm" square>
                        <span class="icon-[fluent--record-stop-20-regular]" />
                      </Button>
                    </Match>
                  </Switch>
                </div>
              </section>
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
