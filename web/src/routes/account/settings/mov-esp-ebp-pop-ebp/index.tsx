import { deleteSelf } from "@/lib/api/account";
import { accountStore, resetUser } from "@/lib/storage/account";
import Button from "@/lib/widgets/button";
import Input from "@/lib/widgets/input";
import { useNavigate } from "@solidjs/router";
import { t } from "@storage/theme";
import Divider from "@widgets/divider";
import { createSignal } from "solid-js";

export default function () {
  const [name, setName] = createSignal("");
  const navigate = useNavigate();
  const canDelete = () => name() === accountStore.account!;
  function handleDeactivate() {
    deleteSelf().then(() => {
      resetUser();
      navigate("/");
    });
  }
  return (
    <div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
      <div class="flex-1 flex flex-col max-w-5xl space-y-2">
        <div class="pt-4 md:p-12 md:pb-4 flex flex-row md:flex-col items-center justify-center">
          <span class="icon-[fluent--warning-24-filled] text-error w-6 h-6 md:w-24 md:h-24" />
          <h1 class="text-center text-lg font-bold text-error ml-4 md:ml-0 md:mt-4">
            {t("account.deleteAccountTitle")}
          </h1>
        </div>
        <Divider class="w-full" />
        <article class="article w-full max-w-5xl self-center mt-4">
          <p>
            <strong>{t("account.deleteAccountTips1")}</strong>
          </p>
          <ul>
            <li>{t("account.deleteAccountTipsList1")}</li>
            <li>{t("account.deleteAccountTipsList2")}</li>
            <li>{t("account.deleteAccountTipsList3")}</li>
            <li>{t("account.deleteAccountTipsList4")}</li>
          </ul>
          <p>
            <strong>{t("account.deleteAccountTips3")}</strong>
          </p>
          <p class="text-error">{t("account.deleteAccountTips2", { name: accountStore.account! })}</p>
        </article>
        <Divider class="w-full" />
        <div class="w-full max-w-5xl self-center flex flex-row items-center justify-center">
          <Input
            icon={<span class="icon-[fluent--person-20-regular] w-5 h-5" />}
            extraBtn={
              <Button class="rounded-l-none text-error" disabled={!canDelete()} onClick={handleDeactivate}>
                <span class="icon-[fluent--arrow-exit-20-regular] w-5 h-5" />
                <span class="icon-[fluent--person-walking-20-regular] w-5 h-5" />
                <span class="hidden md:inline">{t("account.bye")}</span>
              </Button>
            }
            class="flex-1"
            onInput={(v) => {
              setName(v.target.value);
            }}
          />
        </div>
        <Divider class="w-full" />
      </div>
    </div>
  );
}
