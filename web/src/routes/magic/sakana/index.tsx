import "sakana-widget/lib/index.css";
import NewStarBrand from "@assets/brands/NewStar";
import XDSECBrand from "@assets/brands/Ret2Shell";
import newstarMascot from "@assets/imgs/newstar-mascot.webp";
import xdsecMascot from "@assets/imgs/xdsec-mascot.webp";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import Link from "@widgets/link";
import SakanaWidget from "sakana-widget";
import { onMount } from "solid-js";

export default function () {
  const newstar = SakanaWidget.getCharacter("takina");
  const xdsec = SakanaWidget.getCharacter("takina");
  if (newstar) newstar.image = newstarMascot;
  if (xdsec) xdsec.image = xdsecMascot;
  SakanaWidget.registerCharacter("newstar", newstar!);
  SakanaWidget.registerCharacter("xdsec", xdsec!);

  onMount(() => {
    new SakanaWidget({
      character: "newstar",
      controls: false,
      size: 350,
      stroke: {
        color: "#80808060",
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount("#newstar-box");
    new SakanaWidget({
      character: "xdsec",
      controls: false,
      size: 350,
      stroke: {
        color: "#80808060",
        width: 4,
      },
    })
      .setState({ i: 0.05, d: 0.99 })
      .mount("#xdsec-box");
  });
  return (
    <>
      <Title page={t("magic.sakana.title")} route="/magic/sakana" />
      <div class="flex-1 flex flex-row items-center justify-center">
        <div class="relative">
          <div id="newstar-box" />
          <Link
            // background contain
            class="absolute left-1/2 -bottom-12 transform -translate-x-1/2 normal-case z-[10] w-24 h-24"
            href="https://openctf.net"
            target="_blank"
            rel="noopener noreferrer"
          >
            <NewStarBrand width={64} height={64} />
          </Link>
        </div>
        <div class="relative hidden lg:block">
          <div id="xdsec-box" />
          <Link
            class="absolute left-1/2 -bottom-12 transform -translate-x-1/2 normal-case z-[10] w-24 h-24"
            href="https://ctf.xidian.edu.cn"
            target="_blank"
            rel="noopener noreferrer"
          >
            <XDSECBrand width={64} height={64} />
          </Link>
        </div>
      </div>
      <div class="h-24 self-center text-zinc-500 text-center space-x-2">
        <span>{t("magic.sakana.illustration")} By</span>
        <a class="hover:underline" href="https://twitter.com/LAttic1ng" target="_blank" rel="noopener noreferrer">
          Ac4ae0
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
