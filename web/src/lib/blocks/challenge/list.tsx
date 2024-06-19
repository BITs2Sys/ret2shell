import TreeView, { type TreeNode } from "@/lib/widgets/treeview";
import { useSearchParams } from "@solidjs/router";
import { gameStore } from "@storage/game";
import { fullTheme, t } from "@storage/theme";
import Link from "@widgets/link";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { Show, createMemo } from "solid-js";

export default function ChallengeList(props: { showScore?: boolean }) {
    const [searchParams, _] = useSearchParams();
    const selectedChallengeId = createMemo(() => {
        return Number.parseInt(searchParams.challenge || "") ?? null;
    });
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
                    return a.challenge.score - b.challenge.score;
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
                    link: `/games/${gameStore.current?.id}/challenges?challenge=${c.challenge.id}`,
                    extraClasses: c.solved ? "opacity-60" : "",
                    icon: c.solved
                        ? "icon-[fluent--checkmark-circle-20-regular] text-success"
                        : "icon-[fluent--flag-20-regular]",
                    extraPart: props.showScore ? <span class="font-bold">{c.challenge.score} pts</span> : null,
                    children: [],
                })),
            });
        }
        return tree;
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
                        <Show
                            when={gameStore.challenges.length > 0}
                            fallback={
                                <div class="flex flex-row items-center justify-center space-x-2 opacity-60 p-3">
                                    <span class="icon-[fluent--emoji-sad-slight-20-regular] w-5 h-5" />
                                    <span>{t("game.challenge.noChallenges")}</span>
                                </div>
                            }
                        >
                            <TreeView
                                tree={challengesEx()}
                                activeSearchParams="challenge"
                                highlightPaths={
                                    selectedChallengeId()
                                        ? [
                                              selectedChallenge()?.tag.find((t) => t.primary)?.name ??
                                                  t("game.challenge.unknownTag")!,
                                              selectedChallengeId().toString(),
                                          ]
                                        : undefined
                                }
                            />
                        </Show>
                    </div>
                </OverlayScrollbarsComponent>
            </div>
        </>
    );
}
