import type { Article } from "@models/article";
import { Permission } from "@models/user";
import { useNavigate } from "@solidjs/router";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import CreateForm from "./_blocks/form";

export default function () {
  const navigate = useNavigate();
  if (!accountStore.permissions.includes(Permission.Bulletin)) {
    navigate("/sigtrap/403", { replace: true });
  }
  function onDone(article: Article) {
    navigate(`/bulletin/${article.id}`);
  }
  return (
    <>
      <h1 class="text-3xl text-center flex flex-row space-x-4 items-center justify-center font-bold mt-8">
        <span>
          {t("bulletin.create")} - {t("bulletin.title")}
        </span>
      </h1>
      <div class="flex-1 flex flex-col">
        <CreateForm onDone={onDone} />
      </div>
    </>
  );
}
