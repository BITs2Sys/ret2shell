import { AuditList, SubmissionList } from "@blocks/game/lists";
import NarrowTips from "@blocks/narrow-tips";
import { gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Tag from "@widgets/tag";
import { Match, Switch, createSignal } from "solid-js";

export default function () {
  const [tab, setTab] = createSignal("submissions" as "submissions" | "audits");
  return (
    <>
      <Title page={t("game.admin.monitor.title")} route={`/games/${gameStore.current?.id}/admin/monitor`} />
      <div class="w-full p-3 lg:p-6 flex flex-col flex-1 relative">
        <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
          <span class="icon-[fluent--flash-flow-20-regular] w-5 h-5" />
          <span class="flex-1 text-start">{t("game.admin.monitor.title")}</span>
          <Tag level="success">
            <span>{t("game.monitor.autoRefreshEnabled")}</span>
          </Tag>
          <Button
            size="sm"
            ghost={tab() !== "submissions"}
            onClick={() => {
              setTab("submissions");
            }}
          >
            <span class="icon-[fluent--number-symbol-16-regular] w-4 h-4" />
            <span>{t("game.monitor.submissions")}</span>
          </Button>
          <Button
            size="sm"
            ghost={tab() !== "audits"}
            onClick={() => {
              setTab("audits");
            }}
          >
            <span class="icon-[fluent--alert-16-regular] w-4 h-4" />
            <span>{t("game.monitor.audits")}</span>
          </Button>
        </h3>
        <Switch>
          <Match when={tab() === "submissions"}>
            <SubmissionList inGame />
          </Match>
          <Match when={tab() === "audits"}>
            <AuditList />
          </Match>
        </Switch>
        <NarrowTips breakpoint="md" />
      </div>
    </>
  );
}
