import SidebarLayout from "@blocks/sidebar-layout";
import { setGameStore } from "@storage/game";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { type JSX, onCleanup } from "solid-js";
import SideBar from "./_blocks/sidebar";

export default function (props: { children?: JSX.Element }) {
  onCleanup(() => {
    setGameStore({ current: null, games: [], preload: null });
  });
  return (
    <>
      <Title title={`${t("training.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <SidebarLayout leftBar={() => <SideBar />}>{props.children}</SidebarLayout>
    </>
  );
}
