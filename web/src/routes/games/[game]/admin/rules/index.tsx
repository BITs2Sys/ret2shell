import { useUpdateGameDocMutation } from "@api/game";
import GameDocForm from "@routes/games/[game]/_blocks/doc-form";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { createMemo } from "solid-js";

export default function () {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  const updateMutation = useUpdateGameDocMutation();

  async function onSubmit(content: string) {
    await updateMutation.mutateAsync({
      id: gameId(),
      type: "rules",
      content,
    });
  }

  return (
    <>
      <Title page={t("game.rules.title")} route={`/games/${gameId()}/admin/rules`} />
      <div class="flex flex-col p-3 lg:p-6 w-full items-center">
        <GameDocForm gameId={gameId()} docType="rules" onDone={onSubmit} />
      </div>
    </>
  );
}
