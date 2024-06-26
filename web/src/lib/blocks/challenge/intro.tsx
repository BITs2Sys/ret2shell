import { getChallengeAttachments } from "@/lib/api/game";
import type { Challenge } from "@/lib/models/challenge";
import { fullTheme, t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import Article from "@/lib/widgets/article";
import Button from "@/lib/widgets/button";
import type { HTTPError } from "ky";
import { OverlayScrollbarsComponent } from "overlayscrollbars-solid";
import { passiveSupport } from "passive-events-support/src/utils";
import { createEffect, createSignal, For, Show, untrack } from "solid-js";

passiveSupport({
    events: ["mousewheel", "wheel"],
    listeners: [
        {
            element: "challenge-header",
            event: "wheel",
        },
    ],
});

export default function (props: { challenge?: Challenge }) {
    const [larged, setLarged] = createSignal(true);
    const [files, setFiles] = createSignal([] as { folder: "mapped" | "static"; file: string }[]);
    createEffect(() => {
        if (props.challenge) {
            untrack(() => {
                getChallengeAttachments(props.challenge!.game_id, props.challenge!.id)
                    .then((resp) => {
                        setFiles(resp);
                    })
                    .catch((e: HTTPError) => {
                        e.response.text().then((text) => {
                            addToast({
                                level: "error",
                                description: `${t("game.challenge.fetchFilesFailed")}: ${text}`,
                                duration: 5000,
                            });
                        });
                    });
            });
        }
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
                <div
                    class="flex flex-col"
                    onWheel={(e: WheelEvent) => {
                        console.log(e);
                        if (e.deltaY > 0 && larged()) setLarged(false);
                        if (e.deltaY < 0 && !larged()) setLarged(true);
                    }}
                    id="challenge-header"
                >
                    <div class="sticky top-0 w-full flex flex-col border-b border-b-layer-content/15 backdrop-blur z-20">
                        <div
                            class={`flex flex-row self-center w-full max-w-5xl items-center transition-all ${larged() ? "h-36" : "h-16"}`}
                        >
                            <h1 class="flex flex-row items-center">
                                <span
                                    class={`mx-4 icon-[fluent--code-20-regular] transition-all ${larged() ? "w-16 h-16" : "w-5 h-5"}`}
                                />
                                <div class="flex flex-col space-y-2">
                                    <span class={`${larged() ? "text-3xl" : "text-base"} font-bold transition-font`}>
                                        {props.challenge?.name}
                                    </span>
                                    <Show when={larged()}>
                                        <span class={`${larged() ? "h-auto" : "h-0"} overflow-hidden opacity-60`}>
                                            {props.challenge?.updated_at.toFormat("yyyy-MM-dd HH:mm:ss")}
                                        </span>
                                    </Show>
                                </div>
                            </h1>
                        </div>
                    </div>
                    <div class="flex flex-col w-full max-w-5xl self-center p-3 lg:p-6">
                        <Show when={files().length > 0}>
                            <h2 class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold">
                                <span class="icon-[fluent--folder-zip-20-regular] w-5 h-5" />
                                <span>{t("game.challenge.files")}</span>
                            </h2>
                            <div class="flex flex-row flex-wrap p-2">
                                <For each={files()}>
                                    {(file) => (
                                        <Button class="font-normal flex flex-col !h-auto py-2 m-1 overflow-hidden">
                                            <span class="icon-[fluent--folder-zip-20-regular] w-8 h-8 flex-shrink-0" />
                                            <span class="truncate">{file.file}</span>
                                        </Button>
                                    )}
                                </For>
                            </div>
                        </Show>
                        <h2 class="h-12 border-b border-b-layer-content/15 flex flex-row items-center space-x-2 font-bold">
                            <span class="icon-[fluent--document-20-regular] w-5 h-5" />
                            <span>{t("game.challenge.content")}</span>
                        </h2>
                        <Article content={props.challenge?.content ?? ""} extra />
                    </div>
                </div>
            </OverlayScrollbarsComponent>
        </div>
    );
}
