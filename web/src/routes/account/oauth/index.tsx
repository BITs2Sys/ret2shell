import { loginWithOAuth } from "@api/account";
import LogoAnimate from "@assets/animates/logo-animate";
import xdsecMascotHappy from "@assets/imgs/xdsec-mascot-happy.webp";
import { useLocation, useNavigate, useSearchParams } from "@solidjs/router";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import LoadingTips from "@widgets/loading-tips";
import type { HTTPError } from "ky";
import { createSignal, onMount } from "solid-js";

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

function getLogo(provider: string) {
  const logoKeys = Object.keys(logos);
  for (const key of logoKeys) {
    if (provider.startsWith(key)) return logos[key as keyof typeof logos];
  }
  return logo;
}
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
      handleLoginWithOAuth();
    }, 2000);
  });
  const brand = () => {
    const service = searchParams.service;
    if (service) return getLogo(service);
    return logo;
  };

  function handleLoginWithOAuth() {
    loginWithOAuth(location.search)
      .then(() => {
        navigate("/", { replace: true });
        addToast({
          level: "success",
          description: t("account.login.success")!,
          duration: 5000,
          img: xdsecMascotHappy,
        });
      })
      .catch((err: HTTPError) => {
        err.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("account.oauth.failedToLogin")}: ${text}`,
            duration: 5000,
          });
          setTimeout(() => {
            navigate("/account/login", { replace: true });
          });
        });
      });
  }

  return (
    <div class="flex-1 w-full flex flex-col space-y-8 items-center justify-center">
      <div class="flex flex-row space-x-8 items-center">
        <LogoAnimate
          width={128}
          height={128}
          class={`transition-all duration-700 ${animate() ? "" : "translate-x-16 opacity-0"}`}
        />
        <span class={`transition-all duration-700 ${animate() ? "opacity-60" : "translate-x-8 opacity-0"}`}>-*-</span>

        <img
          src={xdsecMascotHappy}
          alt="Broken"
          class={`w-24 h-24 animate-bounce transition-all duration-700 ${animate() ? "" : "translate-y-6 opacity-0"}`}
        />
        <span class={`transition-all duration-700 ${animate() ? "opacity-60" : "-translate-x-8 opacity-0"}`}>-*-</span>

        <img
          src={brand()}
          alt="Brand"
          class={`w-32 h-32 transition-all duration-700 ${animate() ? "" : "-translate-x-16 opacity-0"}`}
        />
      </div>
      <LoadingTips />
    </div>
  );
}
