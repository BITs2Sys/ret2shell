import { getInstitutes, getOAuthStatus, unbindWithOAuth } from "@api/account";
import { getAuthConfig } from "@api/platform";
import type { AuthConfig } from "@models/config";
import type { Institute } from "@models/institute";
import type { OAuth } from "@models/oauth";
import { accountStore } from "@storage/account";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import Link from "@widgets/link";
import Tag from "@widgets/tag";
import type { HTTPError } from "ky";
import { Show, createEffect, createMemo, createSignal, untrack } from "solid-js";

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
  const [institutes, setInstitutes] = createSignal([] as Institute[]);
  const [selfOAuthItems, setSelfOAuthItems] = createSignal([] as OAuth[]);
  getInstitutes()
    .then((items) => setInstitutes(items))
    .catch(() => {});
  function refreshOAuthStatus() {
    getOAuthStatus()
      .then((items) => setSelfOAuthItems(items))
      .catch((err: HTTPError) => {
        setSelfOAuthItems([]);
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("account.settings.oauth.failedToGetOAuthStatus")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  createEffect(() => {
    if (accountStore.token) {
      untrack(() => {
        refreshOAuthStatus();
      });
    }
  });
  const oauthServices = createMemo(() => Object.keys(authConfig().oauth_keys || {}));

  function handleUnbind(id: number) {
    unbindWithOAuth(id)
      .then(() => {
        refreshOAuthStatus();
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("account.oauth.failedToUnbind")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
  return (
    <div class="flex flex-col p-3 lg:p-6 w-full items-center">
      <div class="flex flex-col w-full max-w-5xl relative">
        <h3 class="h-12 flex items-center border-b border-b-layer-content/10 font-bold space-x-2">
          <span class="icon-[fluent--settings-20-regular] w-5 h-5" />
          <span>{t("account.settings.oauth.title")}</span>
        </h3>
        <Show when={oauthServices().find((service) => service === "xdu")}>
          <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10 space-x-2">
            <span class="text-error">XDU</span>
            <h4 class="font-bold text-start flex-1">
              <span>{t("account.oauth.xdu.title")}</span>
            </h4>
            <Show when={institutes().find((v) => v.provider === "xdu")}>
              <Tag level="info">
                <span>{institutes().find((v) => v.provider === "xdu")?.name}</span>
              </Tag>
            </Show>
            <Show
              when={selfOAuthItems().find((v) => v.provider === "xdu")}
              fallback={
                <>
                  <span class="opacity-60">{t("account.oauth.notBind")}</span>
                  <Link
                    size="sm"
                    href={`https://ids.xidian.edu.cn/authserver/login?service=${window.location.origin}/account/bind?service=xdu`}
                  >
                    {t("account.oauth.bind")}
                  </Link>
                </>
              }
            >
              <span class="opacity-60">
                {selfOAuthItems().find((v) => v.provider === "xdu")?.data.name ?? "UNKNOWN"} (
                {selfOAuthItems().find((v) => v.provider === "xdu")?.data.id ?? "UNKNOWN"})
              </span>
              <Button size="sm" onClick={() => handleUnbind(selfOAuthItems().find((v) => v.provider === "xdu")!.id)}>
                {t("account.oauth.unbind")}
              </Button>
            </Show>
          </div>
        </Show>
        <Show when={oauthServices().find((service) => service === "xmu")}>
          <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10 space-x-2">
            <span class="text-info">XMU</span>
            <h4 class="font-bold text-start flex-1">
              <span>{t("account.oauth.xmu.title")}</span>
            </h4>
            <Show when={institutes().find((v) => v.provider === "xmu")}>
              <Tag level="info">
                <span>{institutes().find((v) => v.provider === "xmu")?.name}</span>
              </Tag>
            </Show>
            <Show
              when={selfOAuthItems().find((v) => v.provider === "xmu")}
              fallback={
                <>
                  <span class="opacity-60">{t("account.oauth.notBind")}</span>
                  <Link
                    size="sm"
                    href={`https://ids.xmu.edu.cn/authserver/login?service=${window.location.origin}/account/bind?service=xmu`}
                  >
                    {t("account.oauth.bind")}
                  </Link>
                </>
              }
            >
              <span class="opacity-60">
                {selfOAuthItems().find((v) => v.provider === "xmu")?.data.name ?? "UNKNOWN"} (
                {selfOAuthItems().find((v) => v.provider === "xmu")?.data.id ?? "UNKNOWN"})
              </span>
              <Button size="sm" onClick={() => handleUnbind(selfOAuthItems().find((v) => v.provider === "xmu")!.id)}>
                {t("account.oauth.unbind")}
              </Button>
            </Show>
          </div>
        </Show>
        <Show when={oauthServices().find((service) => service === "jiangnan")}>
          <div class="h-12 flex flex-row items-center border-b border-b-layer-content/10 space-x-2">
            <span class="text-info">JIANGNAN</span>
            <h4 class="font-bold text-start flex-1">
              <span>{t("account.oauth.jiangnan.title")}</span>
            </h4>
            <Show when={institutes().find((v) => v.provider === "jiangnan")}>
              <Tag level="info">
                <span>{institutes().find((v) => v.provider === "jiangnan")?.name}</span>
              </Tag>
            </Show>
            <Show
              when={selfOAuthItems().find((v) => v.provider === "jiangnan")}
              fallback={
                <>
                  <span class="opacity-60">{t("account.oauth.notBind")}</span>
                  <Link size="sm" href={`${window.location.origin}/account/bind?service=jiangnan`}>
                    {t("account.oauth.bind")}
                  </Link>
                </>
              }
            >
              <span class="opacity-60">
                {selfOAuthItems().find((v) => v.provider === "jiangnan")?.data.name ?? "UNKNOWN"} (
                {selfOAuthItems().find((v) => v.provider === "jiangnan")?.data.id ?? "UNKNOWN"})
              </span>
              <Button
                size="sm"
                onClick={() => handleUnbind(selfOAuthItems().find((v) => v.provider === "jiangnan")!.id)}
              >
                {t("account.oauth.unbind")}
              </Button>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}
