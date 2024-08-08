import {
  getChallenge,
  getChallengeAttachments,
  getChallengeEnv,
  getChallengeList,
  getChallengeSolveStatus,
  getSelfSolves,
} from "@api/game";
import type { Challenge, ChallengeEnv } from "@models/challenge";
import type { Submission } from "@models/submission";
import { HTTPError } from "ky";
import { createStore } from "solid-js/store";
import { gameStore } from "./game";
import { t } from "./theme";
import { addToast } from "./toast";

type FileType = "static" | "mapped" | "checker";
type Attachment = { file: string; folder: FileType };

export const [challengeStore, setChallengeStore] = createStore({
  current: null as Challenge | null,
  status: null as {
    solved: boolean;
    solves: number;
  } | null,
  challenges: [] as Challenge[],
  files: [] as Attachment[],
  adminFiles: [] as Attachment[],
  env: null as ChallengeEnv | null,
  solves: [] as Submission[],
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

export function refreshSolves() {
  getSelfSolves(gameStore.current!.id)
    .then((result) => {
      setChallengeStore({ solves: result });
    })
    .catch((e: HTTPError) => {
      addToast({
        level: "error",
        description: `${t("game.challenge.fetchSolvesFailed")}: ${e.response.statusText}`,
        duration: 5000,
      });
    });
}

export function refreshStatus() {
  getChallengeSolveStatus(challengeStore.current!.game_id, challengeStore.current!.id)
    .then((result) => {
      setChallengeStore({ status: result });
    })
    .catch((e: HTTPError) => {
      setChallengeStore({ status: null });
      e.response.text().then((text) => {
        addToast({
          level: "error",
          description: `${t("game.challenge.fetchSolveStatusFailed")}: ${text}`,
          duration: 5000,
        });
      });
    });
}

export function refreshCurrentChallenge() {
  if (challengeStore.current) {
    getChallenge(gameStore.current!.id, challengeStore.current.id)
      .then((resp) => {
        setChallengeStore({ current: resp });
        refreshChallengeAssets();
      })
      .catch((e: HTTPError) => {
        e.response.text().then((text) => {
          addToast({
            level: "error",
            description: `${t("game.challenge.fetchChallengeFailed")}: ${text}`,
            duration: 5000,
          });
        });
      });
  }
}
