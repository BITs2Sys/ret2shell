import { useDeleteGameMutation, useGame } from "@api/game";
import { useNavigate, useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import Divider from "@widgets/divider";
import Input from "@widgets/input";
import { createMemo, createSignal } from "solid-js";

export default function () {
  const [name, setName] = createSignal("");
  const navigate = useNavigate();

  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const game = useGame({ id: gameId, enabled: () => gameId() > 0 });
  const canDelete = () => name() === game.data?.name;

  const deleteMutation = useDeleteGameMutation({
    onSuccess: () => {
      navigate("/games", { replace: true });
    },
  });

  async function handleDeleteGame() {
    if (!game.data) return;
    deleteMutation.mutate({ id: game.data.id });
  }
  return (
    <>
      <Title page={t("game.delete.title")} route={`/games/${gameId()}/admin/delete`} />
      <div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
        <div class="flex-1 flex flex-col max-w-5xl space-y-2">
          <div class="pt-4 md:p-12 md:pb-4 flex flex-row md:flex-col items-center justify-center">
            <span class="shrink-0 icon-[fluent--warning-24-filled] text-error w-6 h-6 md:w-24 md:h-24" />
            <h1 class="text-center text-lg font-bold text-error ml-4 md:ml-0 md:mt-4">{t("game.delete.title")}</h1>
          </div>
          <Divider class="w-full" />
          <article class="article w-full max-w-5xl self-center mt-4">
            <p>
              <strong>{t("game.delete.warning")}</strong>
            </p>
            <p>
              <strong>{t("game.delete.requirements")}</strong>
            </p>
            <p class="text-error">{t("game.delete.confirm", { name: game.data?.name || "" })}</p>
          </article>
          <Divider class="w-full" />
          <Input
            icon={<span class="shrink-0 icon-[fluent--person-20-regular] w-5 h-5" />}
            extraBtn={
              <Button
                class="rounded-l-none text-error"
                disabled={!canDelete() || deleteMutation.isPending}
                onClick={handleDeleteGame}
                loading={deleteMutation.isPending}
              >
                <span class="shrink-0 icon-[fluent--delete-20-regular] w-5 h-5" />
                <span class="hidden md:inline">{t("general.actions.delete.title")}</span>
              </Button>
            }
            class="flex-1 ml-2"
            onInput={(v) => {
              setName(v.target.value);
            }}
          />
          <Divider class="w-full" />
        </div>
      </div>
    </>
  );
}
