import NotImplemented from "@blocks/not-implemented";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { createMemo } from "solid-js";

export default function Capture() {
  const params = useParams();
  const gameId = createMemo(() => Number.parseInt(params.game ?? "", 10) || -1);
  return (
    <>
      <Title page={t("captures.title")} route={`/games/${gameId()}/admin/captures`} />
      <div class="flex-1 flex items-center justify-center">
        <NotImplemented />
      </div>
    </>
  );
}
