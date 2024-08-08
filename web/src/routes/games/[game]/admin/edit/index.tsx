import { updateGame } from "@api/game";
import GameEdit, { type GameForm } from "@blocks/game/form";
import { gameStore, setGameStore } from "@storage/game";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import type { HTTPError } from "ky";
import { DateTime } from "luxon";
import { createSignal } from "solid-js";

export default function () {
  const [loading, setLoading] = createSignal(false);
  function onSubmit(result: GameForm) {
    setLoading(true);
    updateGame(gameStore.current!.id, {
      ...gameStore.current!,
      ...result,
      start_at: DateTime.fromSeconds(result.start_at!),
      end_at: DateTime.fromSeconds(result.end_at!),
      archive_at: DateTime.fromSeconds(result.archive_at!),
      register_at: DateTime.fromSeconds(result.register_at!),
    })
      .then((game) => {
        setGameStore({ current: game });
        addToast({
          level: "success",
          description: t("form.saveSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.saveFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setLoading(false);
      });
  }
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <GameEdit onDone={onSubmit} editSource={gameStore.current || undefined} loading={loading()} inGame />
    </div>
  );
}
