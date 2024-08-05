import { getChallengeAttachments, getChallengeEnv, getChallengeList } from "@api/game";
import type { Challenge, ChallengeEnv } from "@models/challenge";
import { HTTPError } from "ky";
import { createStore } from "solid-js/store";
import { gameStore } from "./game";
import { t } from "./theme";
import { addToast } from "./toast";

type FileType = "static" | "mapped" | "checker";
type Attachment = { file: string; folder: FileType };

export const [challengeStore, setChallengeStore] = createStore({
  current: null as Challenge | null,
  challenges: [] as Challenge[],
  files: [] as Attachment[],
  adminFiles: [] as Attachment[],
  env: null as ChallengeEnv | null,
});

export type ChallengeStoreType = typeof challengeStore;

export async function refreshChallenges() {
  try {
    const result = await getChallengeList(gameStore.current!.id);
    setChallengeStore({ challenges: result[0] });
  } catch (e) {
    const err = e as HTTPError;
    const text = await err.response.text();
    addToast({
      level: "error",
      description: `${t("game.challenge.fetchFailed")}: ${text}`,
      duration: 5000,
    });
  }
}

export async function refreshChallengeAssets() {
  try {
    if (challengeStore.current) {
      const files = await getChallengeAttachments(challengeStore.current.game_id, challengeStore.current.id);
      setChallengeStore({ files });
      const env = await getChallengeEnv(challengeStore.current.game_id, challengeStore.current.id);
      setChallengeStore({ env });
    }
  } catch (e: unknown) {
    if (e instanceof HTTPError) {
      const text = await e.response.text();
      addToast({
        level: "error",
        description: `${t("game.challenge.fetchAssetsFailed")}: ${text}`,
        duration: 5000,
      });
    } else {
      throw e;
    }
  }
}
