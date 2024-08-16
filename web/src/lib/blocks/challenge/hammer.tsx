import { getGamePlayerChatMessages, getTeamSolves, sendGamePlayerChatMessage } from "@api/game";
import Spin from "@assets/animates/spin";
import xdsecMascotCiallo from "@assets/imgs/xdsec-mascot-ciallo.webp";
import { stickerSet } from "@assets/stickers";
import { mediaPath } from "@lib/utils/media";
import type { Challenge } from "@models/challenge";
import type { Chat } from "@models/chat";
import { A } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { challengeStore } from "@storage/challenge";
import { gameStore, isGameAdmin } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Article from "@widgets/article";
import Avatar from "@widgets/avatar";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Editor from "@widgets/editor";
import Popover from "@widgets/popover";
import { HTTPError } from "ky";
import { DateTime } from "luxon";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

export default function (_props: {
  onStateChange?: (challenge?: Challenge) => void;
  inGame?: boolean;
}) {
  const [chats, setChats] = createSignal([] as Chat[]);
  const [chat, setChat] = createSignal("");
  const [sending, setSending] = createSignal(false);
  const [solvedAt, setSolvedAt] = createSignal(null as DateTime | null);

  function handleSendChat() {
    if (chat().trim() === "") return;
    if (gameStore.current && challengeStore.current) {
      setSending(true);
      sendGamePlayerChatMessage(gameStore.current.id, challengeStore.current.id, chat())
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
  }

  function refreshChats() {
    if (gameStore.current && challengeStore.current && !isGameAdmin()) {
      getSolveStatus().then(() => {
        getGamePlayerChatMessages(gameStore.current!.id, challengeStore.current!.id)
          .then((result) => {
            if (result.length > chats().filter((u) => u.id !== 0).length) {
              const last_msg = chats()
                .filter(
                  (u) =>
                    u.user_id !== 0 && u.challenge_id === challengeStore.current?.id && u.team_id === gameStore.team?.id
                )
                .reduce((a, b) => (a.created_at > b.created_at ? a : b), {
                  created_at: DateTime.fromMillis(0),
                });
              setChats([
                ...chats().filter(
                  (u) => u.challenge_id === challengeStore.current?.id && u.team_id === gameStore.team?.id
                ),
                ...result.filter(
                  (u) =>
                    u.created_at > last_msg.created_at &&
                    u.challenge_id === challengeStore.current?.id &&
                    u.team_id === gameStore.team?.id
                ),
              ]);
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

  const interval = setInterval(refreshChats(), 5000);
  onCleanup(() => clearInterval(interval));
  let chatBottomEl: HTMLDivElement;
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
        challenge_id: challengeStore.current?.id!,
        team_id: gameStore.team!.id,
        checked: true,
        game_id: gameStore.current!.id,
      });
    }
    c.sort((a, b) => a.created_at.toMillis() - b.created_at.toMillis());
    return c;
  });

  async function getSolveStatus() {
    if (gameStore.current?.id && gameStore.team?.id && !isGameAdmin()) {
      const resp = await getTeamSolves(gameStore.current.id, gameStore.team.id);
      try {
        const s = resp.find((x) => x.challenge_id === challengeStore.current?.id);
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
  const alreadySend = createMemo(() => chats().at(-1)?.user_id === accountStore.id);

  onMount(() => {
    setTimeout(() => chatBottomEl?.scrollIntoView({ behavior: "smooth" }), 300);
  });

  return (
    <div class="flex flex-col min-h-full relative">
      <div class="flex flex-col flex-1 p-3 lg:p-6 space-y-1">
        <div class="self-start flex-row max-w-[calc(100%-4rem)] flex items-center">
          <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href="/magic/sakana">
            <Avatar class="w-full h-full" src={xdsecMascotCiallo} fallback="Ciallo" />
          </A>
          <div class="w-4 flex-shrink-0" />
          <div class="flex flex-col space-y-1">
            <label class="label">Ciallo～(∠・ω&lt; )⌒☆</label>
            <Card contentClass="p-2">
              <p class="text-wrap">{t("game.challenge.hammerTips")}</p>
            </Card>
            <div class="h-3" />
          </div>
        </div>
        <div class="self-start flex-row max-w-[calc(100%-4rem)] flex items-center">
          <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href="/magic/sakana">
            <Avatar class="w-full h-full" src={xdsecMascotCiallo} fallback="Ciallo" />
          </A>
          <div class="w-4 flex-shrink-0" />
          <div class="flex flex-col space-y-1 items-start">
            <label class="label">Ciallo～(∠・ω&lt; )⌒☆</label>
            <Card contentClass="p-2">
              <p class="text-wrap">
                {t("game.challenge.hammerTips2")}
                {t("game.challenge.hammerTips3")}
              </p>
              <div class="flex flex-row space-x-2 flex-wrap">
                <a
                  class="flex flex-row items-center space-x-1 text-primary hover:underline"
                  href="https://paste.mozilla.org/"
                  target="_blank"
                  rel="noreferrer"
                >
                  <span class="icon-[fluent--earth-20-regular]" />
                  <span>Mozilla Public Pastebin</span>
                </a>
                <a
                  class="flex flex-row items-center space-x-1 text-primary hover:underline"
                  href="https://0x0.st"
                  target="_blank"
                  rel="noreferrer"
                >
                  <span class="icon-[fluent--earth-20-regular]" />
                  <span>0x0.st</span>
                </a>
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
                when={index() === 0 || mixedChats().at(index() - 1)?.user_id !== chat.user_id}
                fallback={<div class="w-10 h-10 flex-shrink-0 self-start" />}
              >
                <Show
                  when={chat.id !== 0}
                  fallback={
                    <A class="w-10 h-10 flex-shrink-0 self-start mt-2" href="/magic/sakana">
                      <Avatar class="w-full h-full" src={xdsecMascotCiallo} fallback="Ciallo" />
                    </A>
                  }
                >
                  <A class="w-10 h-10 flex-shrink-0 self-start" href={`/users/${chat.user_id}`}>
                    <Avatar
                      class="w-full h-full"
                      src={chat.avatar ? mediaPath(chat.avatar) : undefined}
                      fallback={chat.user_name}
                    />
                  </A>
                </Show>
              </Show>
              <div class="w-4 flex-shrink-0" />
              <div class={`flex flex-col space-y-1 ${chat.user_id !== accountStore.id ? "items-start" : "items-end"}`}>
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
                <Card class="peer" contentClass="p-2">
                  <Article content={chat.content} noExtraPaddings compact extra />
                </Card>
                <Show
                  when={index() === mixedChats().length - 1 || mixedChats().at(index() + 1)?.user_id !== chat.user_id}
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
        </div>
        <Editor
          class="h-24 bg-layer"
          value={chat()}
          placeholder={alreadySend() ? t("game.challenge.hammerInputAlreadySend") : t("game.challenge.hammerInput")}
          onKeyPress={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              handleSendChat();
            }
          }}
          lang="markdown"
          onValueChanged={(v) => setChat(v)}
        />
      </div>
      <Show when={sending()}>
        <div class="absolute right-6 bottom-6 lg:right-9 lg:bottom-9">
          <Spin width={20} height={20} />
        </div>
      </Show>
      <div ref={chatBottomEl!} />
      <Show when={isGameAdmin()}>
        <div class="absolute top-0 left-0 w-full h-full bg-layer/60 backdrop-blur flex items-center justify-center">
          <A
            class="font-bold hover:underline hover:text-primary flex items-center space-x-2"
            href={`/games/${gameStore.current?.id}/admin/hammers`}
          >
            <span class="icon-[fluent--open-20-regular] w-5 h-5" />
            <span>{t("game.admin.hammer.shouldGoto")}</span>
          </A>
        </div>
      </Show>
    </div>
  );
}
