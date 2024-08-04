import type { Article } from "@models/article";
import { useNavigate } from "@solidjs/router";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { refreshWikiToc } from "@storage/wiki";
import CreateForm from "../_blocks/form";

export default function () {
  const navigate = useNavigate();
  function onDone(article: Article) {
    void refreshWikiToc().then(() => {
      navigate(`/wiki/${article.id}`);
    });
  }
  return (
    <>
      <Title title={`${t("form.create")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex-1 flex flex-col p-3 lg:p-6">
        <CreateForm onDone={onDone} />
      </div>
    </>
  );
}
