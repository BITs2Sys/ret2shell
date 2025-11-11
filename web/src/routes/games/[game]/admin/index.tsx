import { useNavigate, useParams } from "@solidjs/router";

export default function () {
  const navigate = useNavigate();
  const params = useParams();
  const game_id = Number.parseInt(params.game, 10);

  navigate(`/games/${game_id}/admin/monitor`, { replace: true });
  return null;
}
