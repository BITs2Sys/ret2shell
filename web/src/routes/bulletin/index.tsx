import { useBulletins } from "@api/bulletin";
import Spin from "@assets/animates/spin";
import { randomTips } from "@lib/utils/loading-tips";
import { Permission } from "@models/user";
import { useSearchParams } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Divider from "@widgets/divider";
import Link from "@widgets/link";
import Pagination from "@widgets/pagination";
import clsx from "clsx";
import { createMemo, For, Match, Show, Switch } from "solid-js";

export default function () {
  const [searchParams, setSearchParams] = useSearchParams();
  const page = createMemo(() => (searchParams.page && Number.parseInt(searchParams.page as string, 10)) || 1);
  const pageSize = 15;
  const articles = useBulletins({
    page: () => page(),
    page_size: () => pageSize,
    enabled: () => !!page(),
  });
  return (
    <>
      <Title page={t("bulletin.title")} route="/bulletin" />
      <div class="flex flex-col items-center p-3 lg:p-6 flex-1">
        <div class="flex flex-col flex-1 w-full max-w-5xl">
          <div class="h-12 relative flex flex-row items-center px-4">
            <h1 class="space-x-2 flex flex-row items-center flex-1">
              <span class="shrink-0 icon-[fluent--megaphone-20-regular] w-5 h-5" />
              <span class="font-bold">{t("bulletin.title")}</span>
            </h1>
            <Show when={accountStore.permissions.includes(Permission.Bulletin)}>
              <Link size="sm" level="primary" href="/bulletin/create">
                <span class="shrink-0 icon-[fluent--add-20-regular] w-5 h-5" />
                <span>{t("general.actions.create.title")}</span>
              </Link>
            </Show>
            <Divider class="absolute bottom-0 left-0 w-full" />
          </div>
          <For each={articles.data?.[0]}>
            {(article) => (
              <>
                <Link ghost justify="start" href={`/bulletin/${article.id}`} class="overflow-hidden relative">
                  <span
                    class={clsx(
                      article.weight >= 1 ? "text-primary" : "text-layer-content",
                      article.weight >= 1
                        ? "icon-[fluent--megaphone-20-filled]"
                        : "icon-[fluent--megaphone-20-regular]",
                      "w-5 h-5"
                    )}
                  />
                  <span class="flex-1 text-start truncate font-normal">{article.title}</span>
                  <span class="opacity-60">{article.created_at.toFormat("yyyy-MM-dd")}</span>
                </Link>
                <Divider class="w-full" />
              </>
            )}
          </For>
          <Switch>
            <Match when={articles.data?.[0].length === 0 && !articles.isLoading}>
              <div class="flex-1 flex flex-col items-center justify-center space-y-8 opacity-60">
                <span class="shrink-0 icon-[fluent--megaphone-20-regular] w-24 h-24" />
                <span>{t("bulletin.empty")}</span>
              </div>
            </Match>
            <Match when={articles.isLoading}>
              <div class="flex-1 flex flex-col items-center justify-center space-y-8 opacity-60">
                <Spin width={32} height={32} />
                <span>{randomTips()}</span>
              </div>
            </Match>
          </Switch>
        </div>
        <Pagination
          class="p-6 lg:p-9"
          count={articles.data?.[1] || 0}
          pageSize={pageSize}
          page={page()}
          onPageChange={(page) => setSearchParams({ page: page.page.toString() })}
        />
      </div>
    </>
  );
}
