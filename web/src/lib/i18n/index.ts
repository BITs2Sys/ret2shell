import { type BaseDict, flatten } from "@solid-primitives/i18n";

const localeList = ["zh_cn", "en_us", "zh_tw", "ja_jp"] as const;
export type Locale = (typeof localeList)[number];

export async function fetchDictionary(locale: Locale): Promise<BaseDict> {
  let dict: BaseDict;
  const dictModules = import.meta.glob("./*.json");
  const hyphen = locale.replace("_", "-");
  const match = dictModules[`./${hyphen}.json`];
  try {
    dict = (await match()) as BaseDict;
  } catch {
    dict = await import("./zh-cn.json");
  }
  // flatten the dictionary to make all nested keys available top-level
  const flat = flatten(dict);
  // BITs2CTF fork: overlay fork-owned keys (challenge.fix / koh / isw) from a
  // sibling `fork.<locale>.json` so the upstream locale files stay pristine and
  // never conflict when pulling upstream. Overlay keys win.
  const forkLoader = dictModules[`./fork.${hyphen}.json`];
  if (forkLoader) {
    try {
      const forkFlat = flatten((await forkLoader()) as BaseDict);
      return { ...flat, ...forkFlat };
    } catch {
      // fork overlay missing/broken: fall back to the base dictionary.
    }
  }
  return flat;
}

export function hasLocale(locale: unknown): locale is Locale {
  return typeof locale === "string" && localeList.includes(locale as Locale);
}
