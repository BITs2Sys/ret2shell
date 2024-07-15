import LoadingTips from "@widgets/loading-tips";
import TreeView, { type TreeNode } from "@widgets/treeview";
import { useSearchParams } from "@solidjs/router";
import { gameStore, refreshChallenges } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { Match, Switch, createEffect, createMemo, createSignal, untrack } from "solid-js";

export default function ChallengeList(props: { showScore?: boolean; paginated?: boolean; inGame?: boolean }) {
  const [searchParams, _] = useSearchParams();
  const selectedChallengeId = createMemo(() => {
    return Number.parseInt(searchParams.challenge || "") ?? null;
  });
  const [loading, setLoading] = createSignal(false);
  const selectedChallenge = createMemo(() => gameStore.challenges.find((c) => c.id === selectedChallengeId()));
  const challengesEx = createMemo(() => {
    const result = [];
    for (const challenge of gameStore.challenges) {
      const submission = gameStore.solves.find((s) => s.challenge_id === challenge.id);
      result.push({ challenge, solved: !!submission });
    }
    const tree = [] as TreeNode[];
    const tags = new Set(
      gameStore.challenges.flatMap((c) => c.tag.find((t) => t.primary)?.name ?? t("game.challenge.unknownTag")!)
    );
    for (const tag of tags) {
      const taggedChallenges = result
        .filter((c) => c.challenge.tag.find((t) => t.primary)?.name === tag)
        .sort((a, b) => {
          if (a.solved !== b.solved) {
            return a.solved ? 1 : -1;
          }
          if (a.challenge.score !== b.challenge.score) return a.challenge.score - b.challenge.score;
          return a.challenge.updated_at < b.challenge.updated_at ? -1 : 1;
        });
      tree.push({
        id: tag,
        name: tag,
        type: "category",
        icon: "icon-[fluent--tag-20-regular] w-5 h-5",
        children: taggedChallenges.map((c) => ({
          id: c.challenge.id,
          name: c.challenge.name,
          type: "item",
          searchValue: c.challenge.id.toString(),
          link: props.inGame
            ? `/games/${gameStore.current?.id}/challenges?challenge=${c.challenge.id}`
            : `/training/${gameStore.current?.id}?challenge=${c.challenge.id}`,
          extraClasses: c.solved ? "opacity-60" : "",
          icon: c.challenge.hidden
            ? "icon-[fluent--eye-off-20-regular] w-5 h-5 text-warning"
            : c.solved
              ? "icon-[fluent--checkmark-circle-20-regular] text-success"
              : "icon-[fluent--flag-20-regular]",
          extraPart: props.showScore ? <span class="font-bold opacity-60">{c.challenge.score} pts</span> : null,
          children: [],
        })),
      });
    }
    return tree;
  });
  createEffect(() => {
    if (gameStore.current) {
      untrack(() => {
        // fetch challenges and set them.
        setLoading(true);
        refreshChallenges().finally(() => {
          setLoading(false);
        });
      });
    }
  });
  return (
    <>
      <div class="flex-1 overflow-hidden">
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
          <div class="flex flex-col space-y-2 p-3 lg:p-6">
            <Switch
              fallback={
                <div class="flex flex-row items-center justify-center space-x-2 opacity-60 p-3">
                  <span class="icon-[fluent--emoji-sad-slight-20-regular] w-5 h-5" />
                  <span>{t("game.challenge.noChallenges")}</span>
                </div>
              }
            >
              <Match when={loading()}>
                <div class="flex flex-row items-center justify-center p-3">
                  <LoadingTips />
                </div>
              </Match>

              <Match when={gameStore.challenges.length > 0}>
                <TreeView
                  tree={challengesEx()}
                  activeSearchParams="challenge"
                  highlightPaths={
                    selectedChallengeId()
                      ? [
                          selectedChallenge()?.tag.find((t) => t.primary)?.name ?? t("game.challenge.unknownTag")!,
                          selectedChallengeId().toString(),
                        ]
                      : undefined
                  }
                />
              </Match>
            </Switch>
          </div>
        </OverlayScrollbarsComponent>
      </div>
    </>
  );
}
