import SidebarLayout from "@blocks/sidebar-layout";
import type { Team } from "@models/team";
import { gameStore } from "@storage/game";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { createSignal } from "solid-js";
import Sidebar from "./_blocks/sidebar";

export default function () {
  const [team, setTeam] = createSignal(null as Team | null);
  return (
    <>
      <Title title={`${team()?.name ?? t("game.team.title")} - ${gameStore.current?.name ?? "CTF"}`} />
      <SidebarLayout leftBar={() => <Sidebar />}>{null}</SidebarLayout>
    </>
  );
}
