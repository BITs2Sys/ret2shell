import cumt from "@assets/brands/cumt.svg";
import fudan from "@assets/brands/fudan.svg";
import hdu from "@assets/brands/hdu.svg";
import jiangnan from "@assets/brands/jiangnan.svg";
import jlu from "@assets/brands/jlu.svg";
import seu from "@assets/brands/seu.svg";
import uestc from "@assets/brands/uestc.svg";
import xdu from "@assets/brands/xdu.svg";
import xmu from "@assets/brands/xmu.svg";
import logo from "@assets/logo-gray.svg";

const eduLogos = {
  xdu: xdu,
  xmu: xmu,
  jiangnan: jiangnan,
  hdu: hdu,
  cumt: cumt,
  uestc: uestc,
  seu: seu,
  fudan: fudan,
  jlu: jlu,
};

export function getLogo(provider: string) {
  const logoKeys = Object.keys(eduLogos);
  for (const key of logoKeys) {
    if (provider.startsWith(key)) return eduLogos[key as keyof typeof eduLogos];
  }
  return logo;
}
