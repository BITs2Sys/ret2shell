import { handleHttpError } from "@api";
import { getChallengeCheckerScript, updateChallengeCheckerScript } from "@api/game";
import type { Challenge } from "@models/challenge";
import { challengeStore } from "@storage/challenge";
import { t } from "@storage/theme";
import { addToast } from "@storage/toast";
import Button from "@widgets/button";
import { type DiagnosticMarker, EditorBare } from "@widgets/editor";
import Select from "@widgets/select";
import { createEffect, createMemo, createSignal, untrack } from "solid-js";
import dynamicLeetChecker from "./scripts/dynamic-leet.rx";
import dynamicUuidChecker from "./scripts/dynamic-uuid.rx";
import mappedChecker from "./scripts/mapped.rx";
import simpleChecker from "./scripts/simple.rx";

type PresetChecker = "simple" | "mapped" | "dynamic-leet" | "dynamic-uuid";

const checkerMap = {
  simple: simpleChecker,
  mapped: mappedChecker,
  "dynamic-leet": dynamicLeetChecker,
  "dynamic-uuid": dynamicUuidChecker,
};

// biome-ignore lint/suspicious/noExplicitAny: Context value can be any type
type TmplContext = Record<string, any>;
class Tmpl {
  context: TmplContext;
  constructor(context: TmplContext) {
    this.context = context;
  }

  static withContext(context: TmplContext) {
    return new Tmpl(context);
  }

  // from expression to value
  // biome-ignore lint/suspicious/noExplicitAny: arguments can be any type
  protected handleToken(token: string, callable: boolean, args: any[]) {
    if (!Object.hasOwn(checkerCtx, token)) {
      throw new Error(`Cannot find token in context: ${token}`);
    }
    if (callable) return this.context[token].apply(checkerCtx, args);
    return this.context[token];
  }

  // biome-ignore lint/suspicious/noExplicitAny: everything can income and outcome as string
  private result2str(result: any) {
    if (result === null || typeof result === "undefined") return String(result);
    if (Object.hasOwn(result, "toString")) return result.toString();
    if (Object.hasOwn(Object.getPrototypeOf(result), "toString")) return result.toString();
    return String(result);
  }

  // replace tokens in %token% format
  execute(tmpl: string) {
    const reg = /%([a-zA-Z_]\w*)(\((.*?)\))?%/g;
    return tmpl.replace(reg, (_match, token: string, _callable?: string, _args?: string) => {
      try {
        return this.result2str(this.handleToken(token, !!_callable, _args ? JSON.parse(`[${_args}]`) : []));
      } catch (_err) {
        // console.error(err);
        return _match;
      }
    });
  }
}

const checkerCtx = {
  RANDSTR(n: number) {
    const alphabet = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    return Array.from({ length: n }, () => alphabet[Math.floor(Math.random() * alphabet.length)]).join("");
  },
  RANDREADABLE() {
    const verbs = [
      "building",
      "creating",
      "developing",
      "designing",
      "implementing",
      "optimizing",
      "refactoring",
      "debugging",
      "deploying",
      "testing",
      "monitoring",
      "scaling",
      "exploring",
      "discovering",
      "learning",
      "mastering",
      "crafting",
      "solving",
      "connecting",
      "streaming",
      "processing",
      "analyzing",
      "integrating",
      "automating",
    ];

    const adjectives = [
      "efficient",
      "robust",
      "scalable",
      "modern",
      "elegant",
      "innovative",
      "creative",
      "responsive",
      "dynamic",
      "flexible",
      "secure",
      "lightweight",
      "powerful",
      "smart",
      "intuitive",
      "seamless",
      "reliable",
      "advanced",
      "cutting_edge",
      "user_friendly",
      "high_performance",
      "cloud_native",
      "real_time",
      "cross_platform",
    ];

    // technical nouns
    const techNouns = [
      "API",
      "database",
      "framework",
      "library",
      "component",
      "service",
      "module",
      "pipeline",
      "workflow",
      "architecture",
      "infrastructure",
      "platform",
      "solution",
      "algorithm",
      "protocol",
      "interface",
      "dashboard",
      "analytics",
      "microservice",
      "container",
      "cluster",
      "network",
      "authentication",
      "middleware",
      "cache",
    ];

    // life nouns
    const lifeNouns = [
      "morning_routine",
      "coffee_break",
      "weekend_project",
      "garden_space",
      "kitchen_setup",
      "reading_corner",
      "workout_session",
      "movie_night",
      "dinner_party",
      "beach_walk",
      "hiking_trail",
      "city_exploration",
      "art_gallery",
      "music_festival",
      "book_club",
      "cooking_experiment",
      "travel_adventure",
      "sunset_viewing",
      "stargazing_session",
    ];

    // contexts/domains
    const contexts = [
      "development",
      "production",
      "testing",
      "staging",
      "deployment",
      "monitoring",
      "performance",
      "security",
      "scalability",
      "maintenance",
      "integration",
      "automation",
      "experience",
      "lifestyle",
      "wellness",
      "creativity",
      "productivity",
      "entertainment",
      "education",
      "collaboration",
      "innovation",
      "exploration",
      "relaxation",
      "adventure",
    ];

    // connectors and prepositions
    const connectors = ["with", "for", "in", "on", "through", "using", "via", "across", "within"];

    const patterns = [
      // technical patterns
      () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(techNouns)}`,
      () =>
        `${getRandomItem(adjectives)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
      () =>
        `${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
      () => `${getRandomItem(adjectives)}_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_solution`,

      // life patterns
      () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}`,
      () =>
        `${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
      () => `${getRandomItem(verbs)}_perfect_${getRandomItem(lifeNouns)}_experience`,
      () => `daily_${getRandomItem(verbs)}_${getRandomItem(lifeNouns)}_routine`,

      // mixed patterns
      () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(contexts)}_platform`,
      () =>
        `smart_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
      () => `${getRandomItem(adjectives)}_${getRandomItem(verbs)}_${getRandomItem(contexts)}_workflow`,
      () => `next_gen_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_system`,

      // more complex patterns
      () => `${getRandomItem(verbs)}_and_${getRandomItem(verbs)}_${getRandomItem(techNouns)}`,
      () => `${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}_${getRandomItem(connectors)}_modern_life`,
      () =>
        `${getRandomItem(verbs)}_${getRandomItem(contexts)}_${getRandomItem(connectors)}_${getRandomItem(adjectives)}_way`,
      () => `auto_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(contexts)}`,
    ];

    function getRandomItem(array) {
      return array[Math.floor(Math.random() * array.length)];
    }

    const pattern = getRandomItem(patterns);
    return pattern();
  },
} as const;

export default function (_props: { onStateChange?: (challenge?: Challenge) => void; inGame?: boolean }) {
  const [preset, setPreset] = createSignal(null as PresetChecker | null);
  const presetChecker = createMemo(() => {
    if (!preset()) return null;
    return Tmpl.withContext(checkerCtx).execute(checkerMap[preset()!]);
  });
  const [script, setScript] = createSignal("");
  const [lint, setLint] = createSignal([] as DiagnosticMarker[] | null);
  let serverScript = "";
  async function refreshScript() {
    const resp = await getChallengeCheckerScript(challengeStore.current!.game_id, challengeStore.current!.id, true);
    serverScript = resp.script;
    setScript(resp.script);
    setLint(resp.lint ?? null);
  }
  function restoreScript() {
    setScript(serverScript);
  }
  refreshScript();
  createEffect(() => {
    if (!challengeStore.current?.hidden) {
      untrack(refreshScript);
    }
  });

  createEffect(() => {
    if (presetChecker()) {
      untrack(() => {
        setScript(presetChecker()!);
      });
    }
  });

  async function handleUpdateScript() {
    try {
      await updateChallengeCheckerScript(challengeStore.current!.game_id, challengeStore.current!.id, script());
      addToast({
        level: "success",
        description: t("general.actions.save.status.success")!,
        duration: 5000,
      });
      refreshScript();
    } catch (err) {
      handleHttpError(err as Error, t("general.actions.save.status.fail")!);
    }
  }

  return (
    <div class="flex-1 flex flex-col h-full space-y-2 p-3 lg:p-6 lg:pb-3">
      <header class="min-h-12 border-b border-b-layer-content/10 flex flex-row flex-wrap justify-end space-x-2 items-center gap-y-2 py-2">
        <span class="flex flex-row space-x-2 items-center overflow-hidden">
          <span class="shrink-0 icon-[fluent--code-20-regular] w-5 h-5" />
          <span class="font-bold inline-block whitespace-nowrap">{t("challenge.checker.script")}</span>
          <span class="opacity-60 truncate">checker/main.rx</span>
        </span>
        <span class="flex-1" />
        <span class="flex flex-row justify-end items-center flex-wrap gap-y-2 gap-x-2">
          <Select
            class="w-60 min-w-10"
            placeholder={t("challenge.checker.preset.placeholder")}
            size="sm"
            items={[
              {
                label: t("challenge.checker.preset.simple.label")!,
                value: "simple",
                icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
              },
              {
                label: t("challenge.checker.preset.leet.label")!,
                value: "dynamic-leet",
                icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
              },
              {
                label: t("challenge.checker.preset.uuid.label")!,
                value: "dynamic-uuid",
                icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
              },
              {
                label: t("challenge.checker.preset.mapped.label")!,
                value: "mapped",
                icon: "icon-[fluent--number-symbol-20-regular] w-5 h-5",
              },
            ]}
            onValueChange={(e) => {
              setPreset((e.value.at(0) as PresetChecker) || null);
            }}
          />
          <span class="flex flex-row justify-end items-center flex-wrap gap-y-2 gap-x-2">
            <Button size="sm" square onClick={restoreScript}>
              <span class="shrink-0 icon-[fluent--arrow-reset-20-regular] w-5 h-5" />
            </Button>
            <Button level="info" size="sm" onClick={handleUpdateScript}>
              {t("general.actions.save.title")}
              <span>&</span>
              {t("general.actions.compile.title")}
            </Button>
          </span>
        </span>
      </header>
      <EditorBare
        class="w-full h-full"
        lineNumbers
        lang="rust"
        value={script()}
        lints={lint() ?? []}
        onValueChanged={(e) => {
          setScript(e);
        }}
      />
      <footer class="min-h-12 border-t border-t-layer-content/10 flex flex-col lg:flex-row flex-wrap justify-start space-x-2 items-center gap-y-2 py-2">
        <span class="text-primary icon-[fluent--info-16-regular]" />
        <span class="text-primary">{lint()?.filter((v) => v.kind === "info").length ?? 0}</span>
        <span class="text-warning icon-[fluent--warning-16-regular]" />
        <span class="text-warning">{lint()?.filter((v) => v.kind === "warning").length ?? 0}</span>
        <span class="text-error icon-[fluent--warning-16-regular]" />
        <span class="text-error">{lint()?.filter((v) => v.kind === "error").length ?? 0}</span>
        <div class="flex-1" />
        <a href="https://rune-rs.github.io/" class="text-primary hover:underline">
          Rune Grammar <span class="icon-[fluent--open-12-regular]" />
        </a>
        <span>&nbsp;&nbsp;</span>
        <a href="https://github.com/ret2shell/ret2script" class="text-primary hover:underline">
          Ret2Script <span class="icon-[fluent--open-12-regular]" />
        </a>
      </footer>
    </div>
  );
}
