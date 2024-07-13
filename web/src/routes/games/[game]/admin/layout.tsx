import SidebarLayout from "@blocks/sidebar-layout";
import { gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import type { JSX } from "solid-js";
import SideBar from "./_blocks/sidebar";

export default function (props: { children?: JSX.Element }) {
  return (
    <>
      <Title title={`${t("game.admin.title")} - ${gameStore.current?.name || "CTF"}`} />
      <SidebarLayout leftBar={() => <SideBar />}>{props.children}</SidebarLayout>
    </>
  );
}
