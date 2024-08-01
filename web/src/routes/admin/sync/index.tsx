import NotImplemented from "@blocks/not-implemented";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";

export default function () {
  return (
    <>
      <Title title={`${t("admin.sync.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex-1 flex items-center justify-center">
        <NotImplemented />
      </div>
    </>
  );
}
