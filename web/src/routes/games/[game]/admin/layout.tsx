import SidebarLayout from "@blocks/sidebar-layout";
import { useNavigate } from "@solidjs/router";
import { gameStore, isGameAdmin } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { type JSX, createEffect } from "solid-js";
import SideBar from "./_blocks/sidebar";

export default function (props: { children?: JSX.Element }) {
  const navigate = useNavigate();
  createEffect(() => {
    if (gameStore.current) {
      if (!isGameAdmin()) {
        navigate("/sigtrap/403");
        return null;
      }
    }
  });
  return (
    <>
      <Title title={`${t("game.admin.title")} - ${gameStore.current?.name || "CTF"}`} />
      <SidebarLayout leftBar={() => <SideBar />}>{props.children}</SidebarLayout>
    </>
  );
}
