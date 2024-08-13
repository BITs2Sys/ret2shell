import { getGameAdminChatSessions } from "@api/game";
import type { ChatSession } from "@models/chat";
import { useSearchParams } from "@solidjs/router";
import { gameStore } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Avatar from "@widgets/avatar";
import Divider from "@widgets/divider";
import Link from "@widgets/link";
import type { HTTPError } from "ky";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, Show, createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import { TransitionGroup } from "solid-transition-group";

export default function ChatList() {
  const [sessions, setSessions] = createSignal([] as ChatSession[]);
  const [searchParams, _] = useSearchParams();
  const teamId = createMemo(() => Number.parseInt(searchParams.team ?? "") || null);
  const challengeId = createMemo(() => Number.parseInt(searchParams.challenge ?? "") || null);
  function refreshChats() {
    if (gameStore.current) {
      getGameAdminChatSessions(gameStore.current!.id)
        .then((resp) => {
          // only append new sessions
          const oldSessions = sessions().filter(
            (session) =>
              !resp.find(
                (s) =>
                  s.team_id === session.team_id &&
                  s.challenge_id === session.challenge_id &&
                  (s.last_active_at.toMillis() > session.last_active_at.toMillis() ||
                    s.checked !== session.checked ||
                    s.is_admin !== session.is_admin)
              )
          );
          const newSessions = [
            ...oldSessions,
            ...resp.filter(
              (session) =>
                !oldSessions.find((s) => s.team_id === session.team_id && s.challenge_id === session.challenge_id)
            ),
          ].sort((a, b) => b.last_active_at.toMillis() - a.last_active_at.toMillis());
          setSessions(newSessions);
        })
        .catch((err: HTTPError) => {
          err.response.text().then((text) => {
            addToast({
              level: "error",
              description: `${t("game.challenge.fetchChatSessionError")}: ${text}`,
              duration: 5000,
            });
          });
        });
    }
  }

  const timer = setInterval(() => {
    refreshChats();
  }, 5000);

  onCleanup(() => clearInterval(timer));

  createEffect(() => {
    if (gameStore.current) {
      untrack(refreshChats);
    }
  });
  return (
    <div class="w-full h-full overflow-hidden flex flex-col">
      <div class="h-16 flex flex-row px-4 space-x-2 items-center backdrop-blur border-b border-b-layer-content/10">
        <span class="icon-[fluent--chat-20-regular] w-5 h-5" />
        <span>
          <span class="font-bold">Rx</span>
          <span class="opacity-60">::</span>
          <span>Messenger</span>
        </span>
      </div>
      <OverlayScrollbarsComponent
        class="w-full flex-1 relative"
        options={{
          scrollbars: {
            theme: `os-theme-${fullTheme()}`,
            autoHide: "scroll",
          },
        }}
        defer
      >
        <Show
          when={sessions().length > 0}
          fallback={
            <div class="w-full min-h-full flex flex-row space-x-2 p-3 lg:p-6 items-center justify-center">
              <span class="icon-[fluent--chat-20-regular] w-5 h-5" />
              <span class="font-bold">{t("game.admin.chat.noSession")}</span>
            </div>
          }
        >
          <div class="w-full min-h-full overflow-hidden flex flex-col space-y-2 p-2">
            <TransitionGroup name="fade-group-right">
              <For each={sessions()}>
                {(session) => (
                  <Link
                    href={`/games/${gameStore.current?.id}/admin/hammers?challenge=${session.challenge_id}&team=${session.team_id}`}
                    ghost={!(teamId() === session.team_id && challengeId() === session.challenge_id)}
                    class="flex-row space-x-2 items-center h-auto py-2 !px-3 fade-group-right"
                  >
                    <Avatar class="w-10 aspect-square flex-shrink-0" src={undefined} fallback={session.team_name} />
                    <div class="flex-col flex-1">
                      <div class="flex flex-row space-x-2 items-center">
                        <span class="flex-1 truncate font-bold text-start w-0">{session.team_name}</span>
                        <span class="flex-1 w-0 text-end truncate">{session.challenge_name}</span>
                      </div>
                      <Divider />
                      <div class="flex flex-row space-x-2 items-center overflow-hidden">
                        <Show when={!session.checked && !session.is_admin}>
                          <span class="text-error">[{t("game.admin.chat.unread")}]</span>
                        </Show>
                        <Show when={!session.checked && session.is_admin}>
                          <span class="text-info">[{t("game.admin.chat.playerUnread")}]</span>
                        </Show>
                        <Show when={session.checked && !session.is_admin}>
                          <span class="text-warning">[{t("game.admin.chat.notReply")}]</span>
                        </Show>
                        <span class="flex-1 w-0 truncate opacity-60 text-start font-normal">
                          {session.last_message}
                        </span>
                        <span class="opacity-60">{session.last_active_at.toFormat("MM-dd HH:mm")}</span>
                      </div>
                    </div>
                  </Link>
                )}
              </For>
            </TransitionGroup>
          </div>
        </Show>
      </OverlayScrollbarsComponent>
    </div>
  );
}
