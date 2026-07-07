// BITs2CTF fork: challenge-kind tab wiring (fix / koh / isw), extracted from the
// hot index.tsx dispatch so the fork's footprint there is a single import + a
// `...forkPages` spread + one `<ForkChallengeTabs/>` element — minimising merge
// conflicts on that upstream file.
import { useChallengeAwd, useChallengeAwdp } from "@api/awd";
import { useChallengeFix, useChallengeKoh } from "@api/challenge";
import { useChallengeIsw } from "@api/isw";
import type { Game } from "@models/game";
import { isAdminOfGame } from "@storage/game";
import { t } from "@storage/theme";
import Button from "@widgets/button";
import { Show } from "solid-js";
import Awd from "./awd";
import Awdp from "./awdp";
import Fix from "./fix";
import Isw from "./isw";
import Koh from "./koh";

/// Fork challenge-kind panels, spread into the BottomPanel `pages` map.
export const forkPages = { fix: Fix, koh: Koh, isw: Isw, awd: Awd, awdp: Awdp };

/// Fork challenge-kind tab buttons (shown when admin, or the kind is enabled).
export function ForkChallengeTabs(props: {
  gameId: number;
  challengeId: number;
  game: Game | undefined;
  page: string;
  setTab: (tab: string) => void;
}) {
  const fix = useChallengeFix({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  const koh = useChallengeKoh({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  const isw = useChallengeIsw({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  const awd = useChallengeAwd({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  const awdp = useChallengeAwdp({
    game_id: () => props.gameId,
    challenge_id: () => props.challengeId,
  });
  return (
    <>
      <Show when={isAdminOfGame(props.game) || fix.data?.config?.enabled}>
        <Button onClick={() => props.setTab("fix")} ghost={props.page !== "fix"}>
          <span class="shrink-0 icon-[fluent--wrench-20-regular] w-5 h-5" />
          <span>{t("challenge.fix.title")}</span>
        </Button>
      </Show>
      <Show when={isAdminOfGame(props.game) || koh.data?.config?.enabled}>
        <Button onClick={() => props.setTab("koh")} ghost={props.page !== "koh"}>
          <span class="shrink-0 icon-[fluent--crown-20-regular] w-5 h-5" />
          <span>{t("challenge.koh.title")}</span>
        </Button>
      </Show>
      <Show when={isAdminOfGame(props.game) || isw.data?.enabled}>
        <Button onClick={() => props.setTab("isw")} ghost={props.page !== "isw"}>
          <span class="shrink-0 icon-[fluent--shield-20-regular] w-5 h-5" />
          <span>{t("challenge.isw.title")}</span>
        </Button>
      </Show>
      <Show when={isAdminOfGame(props.game) || awd.data?.config?.enabled}>
        <Button onClick={() => props.setTab("awd")} ghost={props.page !== "awd"}>
          <span class="shrink-0 icon-[fluent--target-arrow-20-regular] w-5 h-5" />
          <span>{t("challenge.awd.title")}</span>
        </Button>
      </Show>
      <Show when={isAdminOfGame(props.game) || awdp.data?.config?.enabled}>
        <Button onClick={() => props.setTab("awdp")} ghost={props.page !== "awdp"}>
          <span class="shrink-0 icon-[fluent--shield-badge-20-regular] w-5 h-5" />
          <span>{t("challenge.awdp.title")}</span>
        </Button>
      </Show>
    </>
  );
}
