import rxSticker from "@assets/imgs/rx.webp";
import { t } from "@storage/theme";

export default function NotImplemented() {
  const randomTips = [
    t("platform.notImplementedTips"),
    t("platform.notImplementedTips1"),
    t("platform.notImplementedTips2"),
    t("platform.notImplementedTips3"),
  ];
  return (
    <div class="flex flex-col space-y-8 items-center justify-center">
      <img class="rounded-xl" src={rxSticker} alt=">ω<" width={256} height={256} />
      <h1 class="font-bold text-3xl space-x-4">
        <span class="opacity-60">{t("platform.hello")}</span>
        <span class="text-primary">|</span>
        <span>{t("platform.notImplemented")}</span>
      </h1>
      <p class="opacity-60">{randomTips[Math.floor(Math.random() * randomTips.length)]}</p>
    </div>
  );
}
