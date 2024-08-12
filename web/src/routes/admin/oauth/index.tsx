import { createInstitute, deleteInstitute, updateInstitute } from "@api/account";
import { getAuthConfig } from "@api/platform";
import type { AuthConfig } from "@models/config";
import type { Institute } from "@models/institute";
import { accountStore, refreshInstitutes } from "@storage/account";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Card from "@widgets/card";
import Dialog from "@widgets/dialog";
import Popover from "@widgets/popover";
import type { HTTPError } from "ky";
import { For, Show, createMemo, createSignal } from "solid-js";
import InstituteForm from "./_blocks/form";

import hdu from "@assets/brands/hdu.svg";
import jiangnan from "@assets/brands/jiangnan.svg";
import xdu from "@assets/brands/xdu.svg";
import xmu from "@assets/brands/xmu.svg";
import cumt from "@assets/brands/cumt.svg";
import logo from "@assets/logo-gray.svg";
const logos = {
  xdu: xdu,
  xmu: xmu,
  jiangnan: jiangnan,
  hdu: hdu,
  cumt: cumt,
};

export default function () {
  const [authConfig, setAuthConfig] = createSignal({
    signing_key: "",
    buffer_time: 0,
    expires_time: 0,
    oauth_keys: {},
  } as AuthConfig);
  getAuthConfig()
    .then((config) => setAuthConfig(config))
    .catch(() => {});
  function getLogo(provider: string) {
    const logoKeys = Object.keys(logos);
    for (const key of logoKeys) {
      if (provider.startsWith(key)) return logos[key as keyof typeof logos];
    }
    return logo;
  }
  const oauthServices = createMemo(() => Object.keys(authConfig().oauth_keys || {}));
  const [loading, setLoading] = createSignal(true);
  refreshInstitutes().then(() => setLoading(false));
  function handleUpdateInstitute(result: Institute) {
    setLoading(true);
    updateInstitute(result)
      .then(() => {
        refreshInstitutes();
        addToast({
          level: "success",
          description: t("form.saveSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.saveFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setLoading(false));
  }
  function handleCreateInstitute(result: Institute) {
    setLoading(true);
    createInstitute(result)
      .then(() => {
        refreshInstitutes();
        addToast({
          level: "success",
          description: t("form.createSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.createFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setLoading(false));
  }
  function handleDeleteInstitute(result: Institute) {
    setLoading(true);
    deleteInstitute(result.id)
      .then(() => {
        refreshInstitutes();
        addToast({
          level: "success",
          description: t("form.deleteSuccess")!,
          duration: 5000,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("form.deleteFailed")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => setLoading(false));
  }
  return (
    <>
      <Title title={`${t("admin.oauth.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex-1 flex flex-col items-center p-3 lg:p-6">
        <div class="w-full max-w-5xl flex flex-col">
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
            <span>{t("admin.oauth.title")}</span>
          </h3>
          <div class="py-2 w-full">
            <Card level="warning" contentClass="p-2 flex flex-row space-x-2 items-center">
              <span class="icon-[fluent--warning-20-regular] w-5 h-5" />
              <span>{t("admin.oauth.warningChangeInConfig")}</span>
            </Card>
          </div>
          <For each={oauthServices()}>
            {(service) => (
              <div class="h-12 flex items-center border-b border-b-layer-content/10 space-x-2">
                <img src={getLogo(service)} alt={service.toUpperCase()} class="w-5 h-5" />
                <h4 class="font-bold text-start flex-1">
                  {/* @ts-expect-error key is dynamic */}
                  <span>{t(`account.oauth.${service}.title`) as string}</span>
                </h4>
                <span class="text-success">{t("admin.oauth.configured")}</span>
              </div>
            )}
          </For>
          <div class="h-36" />
          <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
            <span class="icon-[fluent--hat-graduation-20-regular] w-5 h-5" />
            <span class="flex-1 text-start">{t("admin.institute.title")}</span>
            <Dialog
              level="primary"
              size="sm"
              title={t("form.create")}
              btnContent={
                <>
                  <span class="icon-[fluent--add-20-regular] w-5 h-5" />
                  <span>{t("form.create")}</span>
                </>
              }
            >
              <InstituteForm onDone={handleCreateInstitute} loading={loading()} oauthServices={oauthServices()} />
            </Dialog>
          </h3>
          <For each={accountStore.institutes}>
            {(institute) => (
              <div class="h-12 flex items-center border-b border-b-layer-content/10 space-x-2">
                <span class="icon-[fluent--hat-graduation-20-regular] w-5 h-5" />
                <span class="flex-1 text-start">{institute.name}</span>
                <Show
                  when={institute.provider}
                  fallback={<span class="text-warning px-2">{t("admin.institute.manual")}</span>}
                >
                  <span class="text-success px-2">
                    {t("admin.institute.withOAuth")}: {institute.provider}
                  </span>
                </Show>
                <Dialog
                  ghost
                  size="sm"
                  square
                  title={t("form.edit")}
                  btnContent={<span class="icon-[fluent--edit-20-regular] w-5 h-5" />}
                >
                  <InstituteForm
                    editSource={institute}
                    onDone={handleUpdateInstitute}
                    loading={loading()}
                    oauthServices={oauthServices()}
                  />
                </Dialog>
                <Popover
                  size="sm"
                  ghost
                  square
                  title={t("form.delete")}
                  btnContent={<span class="icon-[fluent--delete-20-regular] w-5 h-5" />}
                >
                  <Card contentClass="p-2 flex flex-row space-x-2 items-center">
                    <span class="icon-[fluent--warning-20-regular] w-5 h-5 text-error" />
                    <span>{t("admin.institute.warningDelete")}</span>
                    <Button
                      level="error"
                      size="sm"
                      title={t("form.confirm")}
                      onClick={() => handleDeleteInstitute(institute)}
                      loading={loading()}
                    >
                      <span>{t("form.confirm")}</span>
                    </Button>
                  </Card>
                </Popover>
              </div>
            )}
          </For>
        </div>
      </div>
    </>
  );
}
