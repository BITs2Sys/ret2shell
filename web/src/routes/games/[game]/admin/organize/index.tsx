import { useInstitutes } from "@api/account";
import { useGame, useUpdateGameMutation } from "@api/game";
import AdministratorsManagement from "@blocks/game/administrators";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Checkbox from "@widgets/checkbox";
import LoadingTips from "@widgets/loading-tips";
import { createMemo, For, Show } from "solid-js";

function InstituteManagement(props: { gameId: number }) {
  const game = useGame({ id: () => props.gameId, enabled: () => props.gameId > 0 });
  const institutes = useInstitutes();
  const updateMutation = useUpdateGameMutation({
    onSuccess: () => {
      game.refetch();
    },
  });
  const loading = createMemo(() => updateMutation.isPending);

  async function handleChangePolicy(restrict: boolean) {
    if (!game.data) return;
    updateMutation.mutate({
      id: game.data.id,
      game: {
        ...game.data,
        access_policy: {
          ...game.data.access_policy,
          restrict,
        },
      },
    });
  }
  async function handleChangeInstitute(institute: number, enabled: boolean) {
    if (!game.data) return;
    const institutes = structuredClone([...(game.data.access_policy.institutes ?? [])]);
    if (enabled) institutes.push(institute);
    else institutes.splice(institutes.indexOf(institute), 1);
    updateMutation.mutate({
      id: game.data.id,
      game: {
        ...game.data,
        access_policy: {
          ...game.data.access_policy,
          institutes,
        },
      },
    });
  }
  return (
    <>
      <Title page={t("game.organize.title")} route={`/games/${props.gameId}/admin/organize`} />
      <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
        <span class="shrink-0 icon-[fluent--settings-20-regular] w-5 h-5" />
        <span>{t("game.organize.title")}</span>
      </h3>
      <Show when={loading()}>
        <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10">
          <LoadingTips />
        </div>
      </Show>
      <div class="grid grid-cols-fit-xs items-center">
        <span class="my-2">{t("game.organize.actions.restrict.confirm")}</span>
        <Checkbox
          checked={game.data?.access_policy.restrict}
          title={t("game.organize.actions.restrict.title")}
          onChange={() => handleChangePolicy(!game.data?.access_policy.restrict)}
        >
          <span class="flex-1 text-start">{t("game.organize.actions.restrict.title")}</span>
        </Checkbox>
      </div>
      <div class="flex flex-col space-y-1">
        <header class="label">{t("game.organize.instituteEnabled")}</header>
        <div class="flex flex-row flex-wrap">
          <For each={institutes.data ?? []}>
            {(institute) => (
              <Checkbox
                class="m-1 flex-none"
                checked={game.data?.access_policy.institutes.includes(institute.id)}
                onChange={() => {
                  handleChangeInstitute(institute.id, !game.data?.access_policy.institutes.includes(institute.id));
                }}
              >
                <span class="flex-1 text-start">{institute.name}</span>
              </Checkbox>
            )}
          </For>
        </div>
      </div>
    </>
  );
}

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center flex-1 min-h-full relative">
      <div class="flex flex-col w-full max-w-5xl relative space-y-2">
        <InstituteManagement gameId={gameId()} />
        <div class="h-12" />
        <AdministratorsManagement gameId={gameId()} />
      </div>
      {/* <NarrowTips breakpoint="md" /> */}
    </div>
  );
}
