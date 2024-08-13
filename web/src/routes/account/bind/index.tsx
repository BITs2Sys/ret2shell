import { bindWithOAuth } from "@api/account";
import LogoAnimate from "@assets/animates/logo-animate";
import xdsecMascotHappy from "@assets/imgs/xdsec-mascot-happy.webp";
import { useLocation, useNavigate, useSearchParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { platformStore } from "@storage/platform";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import LoadingTips from "@widgets/loading-tips";
import type { HTTPError } from "ky";
import { createSignal, onMount } from "solid-js";
import logo from "@assets/logo-gray.svg";
import { getLogo } from "@assets/brands";

export default function () {
  const [animate, setAnimate] = createSignal(false);
  const location = useLocation();
  const navigate = useNavigate();
  const [searchParams, _] = useSearchParams();
  onMount(() => {
    setTimeout(() => {
      setAnimate(true);
    }, 100);
    setTimeout(() => {
      handleBindWithOAuth();
    }, 2000);
  });
  const brand = () => {
    const service = searchParams.service;
    if (service) return getLogo(service);
    return logo;
  };

  function handleBindWithOAuth() {
    bindWithOAuth(location.search)
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("account.oauth.failedToBind")}: ${text}`,
            duration: 5000,
          });
        });
      })
      .finally(() => {
        setTimeout(() => {
          navigate("/account/settings/oauth", { replace: true });
        });
      });
  }

  return (
    <div class="flex-1 w-full flex flex-col space-y-8 items-center justify-center">
      <Title title={`${t("account.bind.title")} - ${platformStore.config.name || t("platform.name")}`} />
      <div class="flex flex-row space-x-8 items-center">
        <LogoAnimate
          width={128}
          height={128}
          class={`transition-all duration-700 ${animate() ? "" : "translate-x-10 opacity-0"}`}
        />
        <span class={`transition-all duration-700 ${animate() ? "opacity-60" : "translate-x-6 opacity-0"}`}>-*-</span>

        <img
          src={xdsecMascotHappy}
          alt="Broken"
          class={`w-24 h-24 animate-bounce transition-all duration-700 ${animate() ? "" : "translate-y-6 opacity-0"}`}
        />
        <span class={`transition-all duration-700 ${animate() ? "opacity-60" : "-translate-x-6 opacity-0"}`}>-*-</span>

        <img
          src={brand()}
          alt="Brand"
          class={`w-32 h-32 transition-all duration-700 ${animate() ? "" : "-translate-x-10 opacity-0"}`}
        />
      </div>
      <LoadingTips />
    </div>
  );
}
