import { handleHttpError } from "@api";
import { getGameRepo } from "@api/game";
import { deunicode } from "@api/rpc";
import Spin from "@assets/animates/spin";
import type { ObjectInfo } from "@models/git";
import { A, useSearchParams } from "@solidjs/router";
import { gameStore } from "@storage/game";
import { t } from "@storage/theme";
import Card from "@widgets/card";
import Clipboard from "@widgets/clipboard";
import Divider from "@widgets/divider";
import Link from "@widgets/link";
import clsx from "clsx";
import { DateTime } from "luxon";
import { createEffect, createSignal, For, Show, untrack } from "solid-js";

export default function () {
  const [repoName, setRepoName] = createSignal<string>(gameStore.current?.name || "");
  const [loading, setLoading] = createSignal(false);

  const [searchParams] = useSearchParams();
  const path = () => ((searchParams.path ?? "") as string).trim().replace(/^[/]+|[/]+$/g, "") || ".";
  const [objects, setObjects] = createSignal<ObjectInfo[]>([]);

  createEffect(() => {
    if (gameStore.current) {
      untrack(async () => {
        setRepoName(await deunicode(gameStore.current!.name, true));
      });
    }
  });

  createEffect(() => {
    if (gameStore.current && path()) {
      untrack(async () => {
        setLoading(true);
        try {
          const result = await getGameRepo(gameStore.current!.id, `${path()}/`);
          setObjects(
            result.sort((a, b) => {
              if (a.type === b.type) {
                return a.path.localeCompare(b.path);
              }
              return b.type.localeCompare(a.type);
            })
          );
        } catch (err) {
          handleHttpError(err as Error, t("game.git.errors.fetchRepo.title")!);
        }
        setLoading(false);
      });
    }
  });

  return (
    <div class="flex flex-col w-full max-w-5xl space-y-2 relative">
      <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
        <span class="shrink-0 icon-[fluent--branch-fork-20-regular] w-5 h-5" />
        <span>{t("game.git.title")}</span>
      </h3>
      <Card level="info" contentClass="p-2 flex flex-row space-x-2 items-center">
        <span class="shrink-0 icon-[fluent--info-20-regular] w-5 h-5" />
        <span>{t("game.git.cloneTip")}</span>
      </Card>
      <Card level="warning" contentClass="p-2 flex flex-row space-x-2 items-center">
        <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5" />
        <Show when={gameStore.current?.hidden} fallback={<span>{t("game.git.status.readonly.message")}</span>}>
          <span>{t("game.git.status.writable.message")}</span>
        </Show>
      </Card>
      <Clipboard value={`${window.location.origin}/api/game/${gameStore.current?.id}/repo/${repoName()}.git`} />
      <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
        <span class="shrink-0 icon-[fluent--branch-fork-20-regular] w-5 h-5" />
        <A class="font-bold" href={`/games/${gameStore.current?.id}/admin/git`}>
          {repoName()}
        </A>
        <span>/</span>
        <For each={path().split("/")}>
          {(p) => (
            <>
              <A
                class="text-layer-content/60"
                href={`/games/${gameStore.current?.id}/admin/git?path=${path().split(p)[0]}${p}`}
              >
                {p}
              </A>
              <span>/</span>
            </>
          )}
        </For>
        <div class="flex-1" />
        <Show when={loading()}>
          <Spin width={20} height={20} />
          <span>{t("general.loading.short")}</span>
        </Show>
      </h3>
      <Card level="warning" contentClass="p-2 flex flex-row space-x-2 items-center">
        <span class="shrink-0 icon-[fluent--warning-20-regular] w-5 h-5" />
        <span>{t("game.git.listTip")}</span>
      </Card>
      <div class="flex flex-col">
        <For each={objects()}>
          {(object) => (
            <>
              <Link
                ghost
                justify="start"
                href={`/games/${gameStore.current?.id}/admin/git?path=${object.path}`}
                class={clsx("overflow-hidden relative", loading() && object.path === path() && "!bg-layer-content/5")}
                disabled={loading() || object.type === "blob"}
                title={object.subject || ""}
              >
                <Show
                  when={loading() && object.path === path()}
                  fallback={
                    <span
                      class={clsx(
                        object.type === "blob"
                          ? "icon-[fluent--document-20-regular]"
                          : "icon-[fluent--folder-20-regular]",
                        "w-5 h-5"
                      )}
                    />
                  }
                >
                  <Spin width={20} height={20} />
                </Show>
                <span class="flex-1 text-start truncate font-normal">{object.path.split("/").slice(-1)[0]}</span>
                <span class="text-primary opacity-60">{object.commit}</span>
                <span class="flex-1 text-start truncate opacity-60 font-normal">{object.subject}</span>
                <span class="opacity-60 font-normal">
                  {DateTime.fromSeconds(object.last_modified || 0).toFormat("yyyy-MM-dd HH:mm")}
                </span>
              </Link>
              <Divider class="w-full" />
            </>
          )}
        </For>
      </div>
    </div>
  );
}
