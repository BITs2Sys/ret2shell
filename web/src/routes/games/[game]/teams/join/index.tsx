import { useNavigate } from "@solidjs/router";
import { canParticipate, gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import { createEffect } from "solid-js";

export default function () {
  const navigate = useNavigate();
  createEffect(() => {
    if (gameStore.current && !canParticipate()) {
      addToast({
        level: "warning",
        description: t("game.canNotParticipate")!,
        duration: 5000,
      });
      navigate(`/games/${gameStore.current.id}`, { replace: true });
    }
  });
  return (
    <>
      <Title title={`${t("game.team.join.title")} - ${gameStore.current?.name || "CTF"}`} />
    </>
  );
}
