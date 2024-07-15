import Link from "@widgets/link";
import { useSearchParams } from "@solidjs/router";
import { fullTheme, t } from "@storage/theme";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { createEffect, createMemo, createSignal, For, Show, untrack } from "solid-js";
import { TransitionGroup } from "solid-transition-group";
import type { Challenge } from "@models/challenge";
import { accountStore } from "@storage/account";
import { Permission } from "@models/user";
import Button from "@widgets/button";

export default function Tabs(props: {
  baseUrl: string;
  current: Challenge | null;
  loading?: boolean;
  inGame?: boolean;
}) {
  const [searchParams, setSearchParams] = useSearchParams();
  const selectedChallengeId = createMemo(() => Number.parseInt(searchParams.challenge || "NaN") || null);
  const [challengeHistory, setChallengeHistory] = createSignal<{ id: number; name: string }[]>([]);
  const inCreate = createMemo(() => searchParams.create === "true");
  const inEditGame = createMemo(() => searchParams.edit === "true");
  function appendChallengeHistory(challenge: Challenge) {
    if (challengeHistory().find((c) => c.id === challenge.id)) {
      setTimeout(() => {
        document
          .getElementById(`challenge-${challenge.id}`)
          ?.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
      }, 100);
      return;
    }
    setChallengeHistory([...challengeHistory(), { id: challenge.id, name: challenge.name }]);
    setTimeout(() => {
      document
        .getElementById(`challenge-${challenge.id}`)
        ?.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
    }, 100);
  }
  createEffect(() => {
    if (props.current) {
      untrack(() => {
        appendChallengeHistory(props.current!);
      });
    }
  });
  return (
    <OverlayScrollbarsComponent
      class="w-full h-16 backdrop-blur border-b border-b-layer-content/10 relative"
      options={{
        scrollbars: {
          theme: `os-theme-${fullTheme()}`,
          autoHide: "scroll",
        },
      }}
      defer
    >
      <div class="h-full flex pr-2 py-0 items-center space-x-2 min-w-max w-max">
        <TransitionGroup name="fade-group-dive-left">
          <div class="fade-group-dive-left flex space-x-2 items-center sticky left-0 pl-2 bg-layer z-20">
            <Link
              href={props.baseUrl}
              onClick={() => setSearchParams({ challenge: null })}
              square={challengeHistory().length > 0}
              ghost
              class="transition-all duration-300 overflow-hidden"
              active={selectedChallengeId() === null && inCreate() === false && inEditGame() === false}
            >
              <span class="icon-[fluent--home-20-regular] w-5 h-5" />
              <Show when={challengeHistory().length === 0}>
                <span>{t("game.challenge.welcome")}</span>
              </Show>
            </Link>
            <Show when={accountStore.permissions.includes(Permission.Game)}>
              <Show when={!props.inGame}>
                <Link
                  active={inEditGame()}
                  title={t("game.admin.edit.title")}
                  square={challengeHistory().length > 0}
                  ghost
                  class="transition-all duration-300 overflow-hidden"
                  href={`${props.baseUrl}?edit=true`}
                >
                  <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
                  <Show when={challengeHistory().length === 0}>
                    <span>{t("game.admin.edit.title")}</span>
                  </Show>
                </Link>
              </Show>
              <Link
                active={inCreate()}
                title={t("game.challenge.create")}
                square={challengeHistory().length > 0}
                ghost
                class="transition-all duration-300 overflow-hidden"
                href={`${props.baseUrl}?create=true`}
              >
                <span class="icon-[fluent--add-20-regular] w-5 h-5" />
                <Show when={challengeHistory().length === 0}>
                  <span>{t("game.challenge.create")}</span>
                </Show>
              </Link>
            </Show>
            <Show when={challengeHistory().length > 0}>
              <span class="h-12 w-[1px] bg-layer-content/15" />
            </Show>
          </div>
          <For each={challengeHistory()}>
            {(challenge) => (
              <div class="fade-group-dive-left flex flex-row">
                <Link
                  href={`${props.baseUrl}?challenge=${challenge.id}`}
                  onClick={() => setSearchParams({ challenge: challenge.id })}
                  id={`challenge-${challenge.id}`}
                  active={challenge.id === selectedChallengeId() && inCreate() === false}
                  ghost
                  class="max-w-48 rounded-r-none"
                >
                  <span class="icon-[fluent--code-20-regular] w-5 h-5" />
                  <span class="truncate flex-1 text-left">{challenge.name}</span>
                </Link>
                <Button
                  class={`!rounded-l-none ${challenge.id === selectedChallengeId() && inCreate() === false ? "btn-active" : ""}`}
                  square
                  ghost
                  onClick={() => {
                    if (challenge.id === selectedChallengeId()) setSearchParams({ challenge: null });
                    setChallengeHistory([...challengeHistory().filter((s) => s.id !== challenge.id)]);
                  }}
                >
                  <span class="icon-[fluent--dismiss-20-regular] w-5 h-5 opacity-60" />
                </Button>
              </div>
            )}
          </For>
        </TransitionGroup>
        <Show when={props.loading}>
          <Button class="opacity-60" loading ghost>
            <span>{t("form.loading")}</span>
          </Button>
        </Show>
      </div>
    </OverlayScrollbarsComponent>
  );
}
