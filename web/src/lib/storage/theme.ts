import { fetchDictionary, hasLocale, type Locale } from "@lib/i18n";
import { resolveTemplate, translator } from "@solid-primitives/i18n";
import { makePersisted } from "@solid-primitives/storage";
import { createEffect, createResource } from "solid-js";
import { createStore } from "solid-js/store";

let systemPrefersLocale = (
  window.navigator.language || window.navigator.languages[0]
)
  .replace("-", "_")
  .toLowerCase() as Locale;

if (!hasLocale(systemPrefersLocale)) {
  systemPrefersLocale = "en_us" as Locale;
}

export const [themeStore, setThemeStore] = makePersisted(
  createStore({
    theme: "cyber",
    locale: systemPrefersLocale,
    colorScheme: "dark",
    colorSchemeFollowsSystem: false,
    showBackgroundImg: true,
  }),
  { name: "theme" },
);

export function setTheme(theme: string) {
  setThemeStore({ theme });
}

export function setColorScheme(_colorScheme: "dark" | "light") {
  // setThemeStore({ colorScheme });
}

export function setLocale(locale: Locale) {
  setThemeStore({ locale });
  setTimeout(() => location.reload());
}

export function toggleBackgroundImg() {
  setThemeStore("showBackgroundImg", !themeStore.showBackgroundImg);
}

export function fullTheme() {
  return `${themeStore.theme}-dark`;
}

export function initTheme() {
  createEffect(() => {
    document.documentElement.setAttribute("data-theme", fullTheme());
    document.documentElement.setAttribute("data-style", "dark");
  });

  function onBeforePrint() {
    document.documentElement.setAttribute(
      "data-theme",
      `${themeStore.theme}-light`,
    );
    document.documentElement.setAttribute("data-style", "light");
  }
  function onAfterPrint() {
    document.documentElement.setAttribute("data-theme", fullTheme());
    document.documentElement.setAttribute("data-style", "dark");
  }
  window.onbeforeprint = onBeforePrint;
  window.onafterprint = onAfterPrint;
}

const [dict] = createResource(
  themeStore.locale || systemPrefersLocale,
  fetchDictionary,
);
export const t = translator(dict, resolveTemplate);
export const colorPalette = {
  fg: () => (themeStore.colorScheme === "dark" ? "#eee" : "#121212"),
  primary: "#0991ed",
  secondary: "#bd63c5",
  accent: "#699f08",
  info: "#0991ed",
  success: "#17a750",
  warning: "#db640e",
  error: "#e05864",
};

export const breakpoints = {
  "2xl": "1536px",
  xl: "1280px",
  lg: "1024px",
  md: "768px",
  sm: "640px",
  xs: "480px",
  "2xs": "320px",
} as const;
