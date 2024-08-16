import {
  getChallenge,
  getGameAdminChatMessages,
  getTeamInfo,
  getTeamSolves,
  sendGameAdminChatMessage,
} from "@api/game";
import Spin from "@assets/animates/spin";
import xdsecMascotCiallo from "@assets/imgs/xdsec-mascot-ciallo.webp";
import { stickerSet } from "@assets/stickers";
import { mediaPath } from "@lib/utils/media";
import type { Challenge } from "@models/challenge";
import type { Chat } from "@models/chat";
import type { Team } from "@models/team";
import { A, useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { gameStore } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import Avatar from "@widgets/avatar";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Editor from "@widgets/editor";
import Link from "@widgets/link";
import Popover from "@widgets/popover";
import { HTTPError } from "ky";
import { DateTime } from "luxon";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, Show, createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";

const quickReplies = [
  t("game.challenge.chatQuickReply1"),
  t("game.challenge.chatQuickReply2"),
  t("game.challenge.chatQuickReply3"),
  t("game.challenge.chatQuickReply4"),
];

function mergeChats(challengeId: number, teamId: number, a: Chat[], b: Chat[]): Chat[] {
  const aa = a.filter((x) => x.challenge_id === challengeId && x.team_id === teamId && x.user_id !== 0);
  if (aa.length === 0) return b;
  // merge b into a, append all new chats and update checked chats
  const last_msg = aa.reduce((i, j) => (i.created_at > j.created_at ? i : j), {
    created_at: DateTime.fromMillis(0),
  });
  const new_chats = b.filter((x) => x.created_at > last_msg.created_at);
  const checked_chats = b.filter((x) => x.checked);
  const checked_ids = checked_chats.map((x) => x.id);
  return aa.map((x) => (checked_ids.includes(x.id) ? { ...x, checked: true } : x)).concat(new_chats);
}

export default function () {
  const [searchParams, _] = useSearchParams();
  const teamId = createMemo(() => Number.parseInt(searchParams.team ?? "") || null);
  const challengeId = createMemo(() => Number.parseInt(searchParams.challenge ?? "") || null);
  const [challenge, setChallenge] = createSignal(null as Challenge | null);
  const [team, setTeam] = createSignal(null as Team | null);
  createEffect(() => {
    if (gameStore.current && challengeId()) {
      getChallenge(gameStore.current.id, challengeId()!)
        .then(setChallenge)
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("game.challenge.fetchChallengeFailed")}: ${text}`,
              duration: 5000,
            });
          });
        });
    }
    if (gameStore.current && teamId()) {
      getTeamInfo(gameStore.current.id, teamId()!)
        .then(setTeam)
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("game.team.fetchTeamFailed")}: ${text}`,
              duration: 5000,
            });
          });
        });
    }
  });
  const [chats, setChats] = createSignal([] as Chat[]);
  const [chat, setChat] = createSignal("");
  const [sending, setSending] = createSignal(false);
  const [solvedAt, setSolvedAt] = createSignal(null as DateTime | null);
  function handleSendChat() {
    if (chat().trim() === "") return;
    setSending(true);
    sendGameAdminChatMessage(gameStore.current!.id, challengeId()!, teamId()!, chat())
      .then(() => {
        setChat("");
        refreshChats();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.sendChatError")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setSending(false));
  }
  let chatBottomEl: HTMLDivElement;
  let cachedTeamId: number | null = null;
  let cachedChallengeId: number | null = null;

  function refreshChats() {
    if (gameStore.current && challengeId() && teamId()) {
      getSolveStatus().then(() => {
        getGameAdminChatMessages(gameStore.current!.id, challengeId()!, teamId()!)
          .then((result) => {
            if (
              cachedTeamId !== teamId() ||
              cachedChallengeId !== challengeId() ||
              result.length > chats().filter((u) => u.id !== 0).length
            ) {
              const r = mergeChats(challengeId() ?? 0, teamId() ?? 0, chats(), result);
              setChats([...r]);
              cachedTeamId = teamId();
              cachedChallengeId = challengeId();
              setTimeout(() => chatBottomEl?.scrollIntoView({ behavior: "smooth" }), 300);
            }
          })
          .catch((err: HTTPError) => {
            err.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("game.challenge.fetchChatError")}: ${text}`,
                duration: 5000,
              });
            });
          });
      });
    }
    return refreshChats;
  }

  createEffect(() => {
    if (gameStore.current && challengeId() && teamId()) {
      untrack(refreshChats);
    }
  });
  const mixedChats = createMemo(() => {
    const c = chats();
    if (solvedAt() && !c.find((x) => x.id === 0)) {
      c.push({
        id: 0,
        user_id: 0,
        user_name: "Ciallo～(∠・ω< )⌒☆",
        avatar: undefined,
        content: `${t("game.challenge.chatSolvedMessage")} ٩(๑•̀ω•́๑)۶`,
        created_at: solvedAt()!,
        is_admin: true,
        challenge_id: challengeId()!,
        team_id: teamId()!,
        checked: true,
        game_id: gameStore.current!.id,
      });
    }
    c.sort((a, b) => a.created_at.toMillis() - b.created_at.toMillis());
    return c;
  });

  async function getSolveStatus() {
    if (gameStore.current?.id && teamId()) {
      const resp = await getTeamSolves(gameStore.current.id, teamId()!);
      try {
        const s = resp.find((x) => x.challenge_id === challengeId());
        if (s) {
          setSolvedAt(s.created_at);
        } else {
          setSolvedAt(null);
        }
      } catch (err) {
        if (err instanceof HTTPError) {
          const text = await err.response.text();
          addToast({
            level: "error",
            description: `${t("game.challenge.fetchSolveError")}: ${text}`,
            duration: 5000,
          });
        }
      }
    }
  }

  const interval = setInterval(refreshChats(), 5000);
  onCleanup(() => clearInterval(interval));
  return (
    <div class="flex-1 h-[calc(100vh-4rem)] overflow-hidden flex flex-col">
      <div class="h-16 flex flex-row px-4 space-x-2 items-center backdrop-blur border-b border-b-layer-content/10">
        <span class="icon-[fluent--chat-20-regular] w-5 h-5" />
        <A class="font-bold" href={`/games/${gameStore.current?.id}/teams/${teamId()}`}>
          {team() ? team()?.name : t("game.admin.chat.title")}
        </A>
        <span class="flex-1" />
        <Show when={challenge()}>
          <A
            class="flex flex-row space-x-2 hover:underline items-center"
            href={`/games/${gameStore.current?.id}/challenges?challenge=${challengeId()}`}
          >
            <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
            <span>{challenge()?.name}</span>
          </A>
        </Show>
      </div>
      <OverlayScrollbarsComponent
        class="w-full flex-1 backdrop-blur relative"
        options={{
          scrollbars: {
            theme: `os-theme-${fullTheme()}`,
            autoHide: "scroll",
          },
        }}
        defer
      >
        <Show
          when={teamId() && challengeId()}
          fallback={
            <div class="min-h-full w-full flex items-center justify-center space-x-2">
              <span class="icon-[fluent--chat-20-regular] w-5 h-5" />
              <span class="font-bold">{t("game.admin.chat.selectToShow")}</span>
            </div>
          }
        >
          <div class="flex flex-col min-h-full relative">
            <div class="flex flex-col flex-1 p-3 lg:p-6 space-y-1">
              <div class="self-start flex-row max-w-[calc(100%-4rem)] flex items-center">
                <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href="/magic/sakana">
                  <Avatar class="w-full h-full" src={xdsecMascotCiallo} fallback="Ciallo" />
                </A>
                <div class="w-4 flex-shrink-0" />
                <div class="flex flex-col space-y-1">
                  <label class="label">Ciallo～(∠・ω&lt; )⌒☆</label>
                  <Card contentClass="flex flex-col space-y-2 p-2">
                    <span>{t("game.challenge.adminHammerTips")}</span>
                    <div class="flex flex-row space-x-2">
                      <Link
                        class="!h-auto p-2"
                        href={`/games/${gameStore.current?.id}/challenges?challenge=${challengeId()}`}
                      >
                        <div class="flex flex-row space-x-2 items-center pr-4">
                          <span class="icon-[fluent--code-20-filled] w-8 h-8 m-2" />
                          <div class="flex flex-col items-start">
                            <h3 class="font-bold">{challenge()?.name}</h3>
                            <p class="opacity-60">{challenge()?.score} pts</p>
                          </div>
                        </div>
                      </Link>
                      <Link class="!h-auto p-2" href={`/games/${gameStore.current?.id}/teams/${teamId()}`}>
                        <div class="flex flex-row space-x-2 items-center pr-4">
                          <span class="icon-[fluent--flag-20-filled] w-8 h-8 m-2" />
                          <div class="flex flex-col items-start">
                            <h3 class="font-bold">{team()?.name}</h3>
                            <p class="opacity-60">{team()?.score} pts</p>
                          </div>
                        </div>
                      </Link>
                    </div>
                  </Card>
                  <div class="h-3" />
                </div>
              </div>
              <For each={mixedChats()}>
                {(chat, index) => (
                  <div
                    class={`${chat.user_id !== accountStore.id ? "self-start flex-row" : "self-end flex-row-reverse"} max-w-[calc(100%-4rem)] flex items-center`}
                  >
                    <Show
                      when={chat.id !== 0}
                      fallback={
                        <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href="/magic/sakana">
                          <Avatar class="w-full h-full" src={xdsecMascotCiallo} fallback="Ciallo" />
                        </A>
                      }
                    >
                      <Show
                        when={index() === 0 || mixedChats().at(index() - 1)?.user_id !== chat.user_id}
                        fallback={<div class="w-10 h-10 flex-shrink-0 self-start" />}
                      >
                        <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href={`/users/${chat.user_id}`}>
                          <Avatar
                            class="w-full h-full"
                            src={chat.avatar ? mediaPath(chat.avatar) : undefined}
                            fallback={chat.user_name}
                          />
                        </A>
                      </Show>
                    </Show>
                    <div class="w-4 flex-shrink-0" />
                    <div
                      class={`flex flex-col space-y-1 ${chat.user_id !== accountStore.id ? "items-start" : "items-end"}`}
                    >
                      <Show when={index() === 0 || mixedChats().at(index() - 1)?.user_id !== chat.user_id}>
                        <label class="label space-x-2">
                          <Show when={chat.user_id !== 0}>
                            <Show
                              when={chat.is_admin}
                              fallback={<span class="text-info">[{t("game.challenge.chatPlayerRole")}]</span>}
                            >
                              <span class="text-error">[{t("game.challenge.chatAdminRole")}]</span>
                            </Show>
                          </Show>
                          <A href={`/users/${chat.user_id}`}>{chat.user_name}</A>
                        </label>
                      </Show>
                      <div class={`peer flex ${chat.user_id !== accountStore.id ? "flex-row" : "flex-row-reverse"}`}>
                        <Card contentClass="p-2">
                          <Article content={chat.content} noExtraPaddings compact extra />
                        </Card>
                        <div class="px-2 self-end flex items-end">
                          <span
                            class={
                              chat.checked
                                ? "icon-[fluent--checkmark-circle-20-filled] w-5 h-5 text-success"
                                : "icon-[fluent--circle-20-regular] w-5 h-5 opacity-40"
                            }
                          />
                        </div>
                      </div>
                      <Show
                        when={
                          index() === mixedChats().length - 1 || mixedChats().at(index() + 1)?.user_id !== chat.user_id
                        }
                      >
                        <label class="opacity-0 peer-hover:opacity-60 text-sm transition-all duration-300">
                          {chat.created_at.toFormat("yyyy-MM-dd HH:mm:ss")}
                        </label>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
            <div class="sticky bottom-0 p-3 lg:p-6">
              <div class="h-full w-full relative">
                <Popover
                  class="absolute -top-10 left-2"
                  size="sm"
                  square
                  ghost
                  btnContent={<span class="icon-[fluent--emoji-20-regular] w-5 h-5" />}
                >
                  <Card contentClass="p-2 aspect-square">
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
                      <div class="grid grid-cols-4 gap-2">
                        <For each={stickerSet}>
                          {(sticker) => (
                            <Button
                              ghost
                              class="p-0 aspect-square overflow-hidden"
                              onClick={() => {
                                setChat(`![${sticker.alt}](${sticker.src})`);
                                setTimeout(() => {
                                  handleSendChat();
                                });
                              }}
                            >
                              <img
                                class="w-16 h-16 transition-transform duration-300 hover:scale-[1.1]"
                                src={sticker.src}
                                alt={sticker.alt}
                                title={sticker.alt}
                              />
                            </Button>
                          )}
                        </For>
                      </div>
                    </OverlayScrollbarsComponent>
                  </Card>
                </Popover>
                <Popover
                  class="absolute -top-10 left-12"
                  size="sm"
                  square
                  ghost
                  btnContent={<span class="icon-[fluent--flash-20-regular] w-5 h-5" />}
                >
                  <Card contentClass="p-2">
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
                      <div class="flex flex-col space-y-2">
                        <For each={quickReplies}>
                          {(reply) => (
                            <Button
                              size="sm"
                              justify="start"
                              ghost
                              onClick={() => {
                                setChat(reply!);
                                setTimeout(() => {
                                  handleSendChat();
                                });
                              }}
                            >
                              <span class="icon-[fluent--chat-20-regular] w-5 h-5" />
                              <span>{reply}</span>
                            </Button>
                          )}
                        </For>
                      </div>
                    </OverlayScrollbarsComponent>
                  </Card>
                </Popover>
                <Editor
                  class="h-24 bg-layer"
                  value={chat()}
                  placeholder={t("game.admin.chat.sendPlaceholder")}
                  onKeyPress={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                      handleSendChat();
                    }
                  }}
                  lang="markdown"
                  onValueChanged={(v) => setChat(v)}
                />
              </div>
            </div>
            <Show when={sending()}>
              <div class="absolute right-6 bottom-6 lg:right-9 lg:bottom-9">
                <Spin width={20} height={20} />
              </div>
            </Show>
            <div ref={chatBottomEl!} />
          </div>
        </Show>
      </OverlayScrollbarsComponent>
    </div>
  );
}
