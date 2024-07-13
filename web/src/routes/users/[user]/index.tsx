import { getUser } from "@api/user";
import SidebarLayout from "@blocks/sidebar-layout";
import type { User } from "@models/user";
import { useNavigate, useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import Article from "@widgets/article";
import LoadingTips from "@widgets/loading-tips";
import { Match, Switch, createSignal } from "solid-js";
import Sidebar from "./_blocks/sidebar";

export default function () {
  const [user, setUser] = createSignal(null as null | User);
  const [loading, setLoading] = createSignal(true);
  const params = useParams();
  const navigate = useNavigate();
  const userId = Number.parseInt(params.user) || null;
  if (!userId) {
    navigate("/sigtrap/404", { replace: true });
    return null;
  }
  getUser(userId)
    .then(setUser)
    .finally(() => setLoading(false));
  return (
    <>
      <Title title={`${user()?.nickname} - ${platformStore.config.name || t("platform.name")!}`} />
      <SidebarLayout leftBar={() => <Sidebar user={user()} loading={loading()} />}>
        <div class="flex-1 flex flex-col items-center">
          <div class="flex flex-col w-full max-w-5xl p-3 lg:p-6">
            <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
              <span class="icon-[fluent--person-20-regular] w-5 h-5" />
              <span>{t("user.introductionTitle")}</span>
            </h3>
            <section>
              <Switch>
                <Match when={loading()}>
                  <LoadingTips />
                </Match>
                <Match when={true}>
                  <Article content={user()?.description || t("user.noDescription")!} />
                </Match>
              </Switch>
            </section>
            <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
              <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
              <span>{t("user.joinedGamesTitle")}</span>
            </h3>
          </div>
        </div>
      </SidebarLayout>
    </>
  );
}
