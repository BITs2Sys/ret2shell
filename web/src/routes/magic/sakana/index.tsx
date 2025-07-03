import "sakana-widget/lib/index.css";
import r3kapig from "@assets/brands/r3kapig.webp";
import r3kapigMascot from "@assets/imgs/r3kapig-mascot.webp";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Link from "@widgets/link";
import SakanaWidget from "sakana-widget";
import { onMount } from "solid-js";

export default function () {
  const r3kapigWidget = SakanaWidget.getCharacter("takina");
  if (r3kapigWidget) r3kapigWidget.image = r3kapigMascot;
  SakanaWidget.registerCharacter("r3kapig", r3kapigWidget!);

  onMount(() => {
    new SakanaWidget({
      character: "r3kapig",
      controls: false,
      size: 350,
      stroke: {
        color: "#80808060",
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount("#r3kapig-box");
  });
  return (
    <>
      <Title page={t("magic.sakana.title")} route="/magic/sakana" />
      <div class="flex-1 flex flex-row items-center justify-center">
        <div class="relative block">
          <div id="r3kapig-box" />
          <Link
            class="absolute left-1/2 -bottom-12 transform -translate-x-1/2 normal-case z-[10] w-24 h-24"
            href="https://www.r3kapig.com"
            target="_blank"
            rel="noopener noreferrer"
          >
            <img src={r3kapig} class="w-16 h-16 object-contain" width={64} height={64} alt="R3kapig" />
          </Link>
        </div>
      </div>
      <div class="h-24 self-center text-zinc-500 text-center space-x-2">
        <span>{t("magic.sakana.illustration")} By</span>
        <a class="hover:underline" href="https://xyy9233.github.io/" target="_blank" rel="noopener noreferrer">
          W3nL0u
        </a>
        <br />
        <span>{t("magic.sakana.source")}</span>
        <a class="hover:underline" href="https://lab.magiconch.com/sakana/" target="_blank" rel="noopener noreferrer">
          https://lab.magiconch.com/sakana
        </a>
      </div>
    </>
  );
}
