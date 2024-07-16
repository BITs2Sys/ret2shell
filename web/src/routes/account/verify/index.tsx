import { verifyEmail } from "@/lib/api/account";
import Spin from "@/lib/assets/animates/spin";
import { t } from "@/lib/storage/theme";
import { addToast } from "@/lib/storage/toast";
import xdsecMascotHappy from "@assets/imgs/xdsec-mascot-happy.webp";
import { useNavigate, useSearchParams } from "@solidjs/router";
import type { HTTPError } from "ky";
import { onMount } from "solid-js";

export default function () {
  const [searchParams, _] = useSearchParams();
  const navigate = useNavigate();
  const email = () => searchParams.email;
  const token = () => searchParams.token;
  onMount(() => {
    setTimeout(() => {
      if (email() && token())
        verifyEmail({ email: email()!, token: token()! })
          .then(() => {
            addToast({
              level: "success",
              description: t("account.emailVerified")!,
              duration: 5000,
            });
            navigate("/account/settings", { replace: true });
          })
          .catch((e: HTTPError) => {
            e.response.text().then((text) => {
              addToast({
                level: "error",
                description: `${t("account.emailVerifyFailed")}: ${text}`,
                duration: 5000,
              });
              navigate("/sigtrap/412", { replace: true });
            });
          });
      else {
        addToast({
          level: "error",
          description: t("account.emailVerifyBroken")!,
          duration: 5000,
        });
        navigate("/sigtrap/418", { replace: true });
      }
    }, 1000);
  });
  return (
    <div class="flex-1 flex flex-col items-center justify-center space-y-8">
      <img src={xdsecMascotHappy} alt="Broken" class="w-48 h-48 animate-bounce" />
      <div class="flex flex-row space-x-4 items-center">
        <Spin />
        <span class="font-bold text-xl">{t("account.verifyingEmail")}...</span>
      </div>
    </div>
  );
}
