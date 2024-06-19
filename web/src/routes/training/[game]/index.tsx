import type { Challenge as ChallengeModel } from "@/lib/models/challenge";
import { Permission } from "@/lib/models/user";
import { accountStore } from "@/lib/storage/account";
import { gameStore, setGameStore } from "@/lib/storage/game";
import { fullTheme, t } from "@/lib/storage/theme";
import Link from "@/lib/widgets/link";
import LoadingTips from "@/lib/widgets/loading-tips";
import Challenge from "@blocks/challenge";
import { useSearchParams } from "@solidjs/router";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { For, Match, Show, Switch, createMemo, createSignal, onCleanup } from "solid-js";
import Intro from "../_blocks/intro";

export default function () {
    const [searchParams, setSearchParams] = useSearchParams();
    const inCreate = createMemo(() => searchParams.create === "true");
    const [challengeHistory, setChallengeHistory] = createSignal<{ id: number; name: string }[]>([]);
    function appendChallengeHistory(challenge: ChallengeModel) {
        if (challengeHistory().find((c) => c.id === challenge.id)) {
            return;
        }
        setChallengeHistory([...challengeHistory(), { id: challenge.id, name: challenge.name }]);
    }
    const selectedChallengeId = createMemo(() => Number.parseInt(searchParams.challenge || "NaN") || null);
    const [selectedChallenge, setSelectedChallenge] = createSignal(null as null | ChallengeModel);

    onCleanup(() => {
        setGameStore({ current: null });
    });

    return (
        <div class="flex-1 flex flex-col w-0">
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
                <div class="h-full flex px-2 py-0 items-center space-x-2 min-w-max w-max">
                    <Link
                        href={`/training/${gameStore.current?.id}`}
                        onClick={() => setSearchParams({ challenge: null })}
                        ghost
                        active={selectedChallengeId() === null && inCreate() === false}
                    >
                        <span class="icon-[fluent--home-20-regular] w-5 h-5" />
                        <span>{t("training.challenge.welcome")}</span>
                    </Link>

                    <Show when={accountStore.permissions.includes(Permission.Game)}>
                        <Link
                            active={inCreate()}
                            title={t("form.create")}
                            ghost
                            href={`/training/${gameStore.current?.id}?create=true`}
                        >
                            <span class="icon-[fluent--add-20-regular] w-5 h-5" />
                            <span>{t("form.create")}</span>
                        </Link>
                    </Show>
                    <For each={challengeHistory()}>
                        {(challenge) => (
                            <Link
                                href={`/training/${gameStore.current?.id}?challenge=${challenge.id}`}
                                onClick={() => setSearchParams({ challenge: challenge.id })}
                                active={challenge.id === selectedChallengeId() && inCreate() === false}
                                ghost
                            >
                                <span class="icon-[fluent--code-20-regular] w-5 h-5" />
                                <span>{challenge.name}</span>
                            </Link>
                        )}
                    </For>
                </div>
            </OverlayScrollbarsComponent>
            <Switch fallback={<Intro />}>
                <Match when={selectedChallenge()}>
                    <Challenge inGame={false} challenge={selectedChallenge()!} />
                </Match>
                <Match when={selectedChallengeId() !== null}>
                    <div class="flex-1 flex flex-row space-x-2 items-center justify-center">
                        <LoadingTips />
                    </div>
                </Match>
            </Switch>
        </div>
    );
}
