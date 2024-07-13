import { setGameStore } from "@storage/game";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { type JSX, onCleanup } from "solid-js";
import Cover from "./_blocks/cover";

export default function (props: { children?: JSX.Element }) {
  onCleanup(() => {
    setGameStore({ current: null, games: [], preload: null });
  });
  return (
    <>
      <Title title={`${t("game.title")} - ${platformStore.config.name || t("platform.name")}`} />
      {props.children}
      <Cover />
    </>
  );
}
