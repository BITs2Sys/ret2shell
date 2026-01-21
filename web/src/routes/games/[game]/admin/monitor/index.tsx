import { useGame } from "@api/game";
import { AuditList, SubmissionList } from "@blocks/game/lists";
import { useParams, useSearchParams } from "@solidjs/router";
import { isGameInArchived } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Tag from "@widgets/tag";
import { createMemo, createSignal, Match, Switch } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });

  const [tab, setTab] = createSignal("submissions" as "submissions" | "audits");
  const [_, setSearchParams] = useSearchParams();
  return (
    <>
      <Title page={t("game.monitor.title")} route={`/games/${gameId()}/admin/monitor`} />
      <div class="w-full p-3 lg:p-6 flex flex-col flex-1 relative">
        <h3 class="min-h-12 py-2 gap-y-2 flex flex-wrap justify-end items-center border-b border-b-layer-content/10 font-bold space-x-2 *:whitespace-nowrap">
          <span class="flex flex-row items-center space-x-2">
            <span class="shrink-0 icon-[fluent--flash-flow-20-regular] w-5 h-5" />
            <span class="flex-1 text-start">{t("game.monitor.title")}</span>
          </span>
          <span class="flex-1" />
          <Tag level="success">
            <span>{t("game.monitor.autoRefreshEnabled")}</span>
          </Tag>
          <span class="flex flex-row items-center space-x-2">
            <Button
              size="sm"
              ghost={tab() !== "submissions"}
              onClick={() => {
                setTab("submissions");
                setSearchParams({ page: null });
              }}
            >
              <span class="shrink-0 icon-[fluent--number-symbol-16-regular] w-4 h-4" />
              <span>{t("game.monitor.submissions")}</span>
            </Button>
            <Button
              size="sm"
              ghost={tab() !== "audits"}
              onClick={() => {
                setTab("audits");
                setSearchParams({ page: null });
              }}
            >
              <span class="shrink-0 icon-[fluent--alert-16-regular] w-4 h-4" />
              <span>{t("game.monitor.audits")}</span>
            </Button>
          </span>
        </h3>
        <Switch>
          <Match when={tab() === "submissions"}>
            <SubmissionList gameId={gameId()} archived={isGameInArchived(game.data)} />
          </Match>
          <Match when={tab() === "audits"}>
            <AuditList gameId={gameId()} />
          </Match>
        </Switch>
        {/* <NarrowTips breakpoint="md" /> */}
      </div>
    </>
  );
}
