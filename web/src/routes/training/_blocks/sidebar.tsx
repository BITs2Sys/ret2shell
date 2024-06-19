import { getGame } from "@/lib/api/game";
import ChallengeList from "@/lib/blocks/challenge/list";
import { useNavigate, useParams } from "@solidjs/router";
import { gameStore, setGameStore } from "@storage/game";
import { t } from "@storage/theme";
import Link from "@widgets/link";
import type { HTTPError } from "ky";
import { Show, createEffect, untrack } from "solid-js";
import Playgrounds from "./playgrounds";

export default function SideBar() {
    const params = useParams();
    const selectedGameId = () => Number.parseInt(params.game) ?? null;
    const navigate = useNavigate();
    createEffect(() => {
        if (selectedGameId()) {
            untrack(() => {
                getGame(selectedGameId())
                    .then((resp) => {
                        // console.log(resp);
                        setGameStore({ current: resp });
                    })
                    .catch((err: HTTPError) => {
                        navigate(`/sigtrap/${err.response.status}`, { replace: true });
                    });
            });
        }
    });
    return (
        <div class="flex flex-col overflow-hidden w-full h-full">
            <div class="border-b border-b-layer-content/10 px-2 h-16 flex items-center justify-center">
                <Show
                    when={gameStore.current}
                    fallback={
                        <Link class="w-full" ghost justify="start" href="/training">
                            <span class="icon-[fluent--dumbbell-20-filled] w-5 h-5 text-primary" />
                            <span>{t("training.list")}</span>
                        </Link>
                    }
                >
                    <Link class="w-full" ghost justify="start" href={`/training/${gameStore.current?.id}`}>
                        <span class="icon-[fluent--dumbbell-20-filled] w-5 h-5 text-primary" />
                        <span>{gameStore.current?.name}</span>
                    </Link>
                </Show>
            </div>
            <Show when={selectedGameId()} fallback={<Playgrounds />}>
                <ChallengeList />
            </Show>
        </div>
    );
}
