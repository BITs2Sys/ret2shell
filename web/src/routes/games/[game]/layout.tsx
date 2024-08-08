import { getGame } from "@api/game";
import { HostType } from "@models/game";
import { useNavigate, useParams } from "@solidjs/router";
import { setChallengeStore } from "@storage/challenge";
import { gameStore, refreshSelfTeam, setGameStore } from "@storage/game";
import { Title } from "@storage/header";
import type { HTTPError } from "ky";
import { type JSX, onCleanup } from "solid-js";
import TeamCover from "./_blocks/team-cover";

export default function (props: { children?: JSX.Element }) {
  const navigate = useNavigate();
  onCleanup(() => {
    setGameStore({ current: null, preload: null, team: null, showTeamCover: false });
    setChallengeStore({ current: null, challenges: [], solves: [] });
  });
  const params = useParams();
  const game_id = Number.parseInt(params.game);
  if (game_id) {
    getGame(game_id)
      .then((resp) => {
        if (resp.host_type !== HostType.CTFGame) {
          navigate(`/training/${resp.id}`);
          return null;
        }
        setGameStore({ current: resp });
        setTimeout(() => {
          refreshSelfTeam();
        });
      })
      .catch((err: HTTPError) => {
        navigate(`/sigtrap/${err.response.status}`, { replace: true });
      });
  }
  return (
    <>
      <Title title={gameStore.current?.name || "CTF"} />
      {props.children}
      <TeamCover />
    </>
  );
}
