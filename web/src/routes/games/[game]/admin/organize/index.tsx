import { t } from "@storage/theme";

export default function () {
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <div class="flex flex-col w-full max-w-5xl space-y-2 relative">
        <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span>{t("game.admin.organize.title")}</span>
        </h3>
      </div>
    </div>
  );
}
