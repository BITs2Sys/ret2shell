import { useWikiTree } from "@api/wiki";
import type { Article } from "@models/article";
import { useNavigate } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import CreateForm from "../_blocks/form";

export default function () {
  const navigate = useNavigate();
  const wikiTree = useWikiTree({ enabled: () => false });
  async function onDone(article: Article) {
    await wikiTree.refetch();
    navigate(`/wiki/${article.id}`);
  }
  return (
    <>
      <Title page={t("general.actions.create.title")} route="/wiki/create" />
      <div class="flex-1 flex flex-col p-3 lg:p-6">
        <CreateForm onDone={onDone} />
      </div>
    </>
  );
}
