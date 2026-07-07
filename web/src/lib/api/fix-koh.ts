// BITs2CTF fork: Fix + KoH challenge-kind API hooks, relocated out of the upstream
// api/challenge.ts (re-exported from there) so that hot file stays near-pristine
// for clean upstream merges.
import type { FixConfig, FixStatus, KohConfig, KohEvent, KohScore, KohStatus } from "@models/challenge";
import type { Submission } from "@models/submission";
import { t } from "@storage/theme";
import { useMutation, useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import api, { api_root, handleHttpError, inflyClient, safeJson, toastSuccess } from ".";

export async function getChallengeFix(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/fix`).json<FixStatus>();
}

export function useChallengeFix({
  game_id,
  challenge_id,
  enabled,
  onError,
}: {
  game_id: () => number;
  challenge_id: () => number;
  enabled?: () => boolean;
  onError?: (err: Error) => boolean;
}) {
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "fix"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getChallengeFix(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.fix.errors.fetch.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function updateChallengeFix(game_id: number, challenge_id: number, config: FixConfig) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/fix`, {
      json: config,
    })
    .json<FixConfig>();
}

export function useUpdateChallengeFixMutation(
  props: { onSuccess?: (config: FixConfig) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; config: FixConfig }) =>
      updateChallengeFix(req.game_id, req.challenge_id, req.config),
    onSuccess: (data: FixConfig) => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function deleteChallengeFix(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/fix`).json<void>());
}

export function useDeleteChallengeFixMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void }) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => deleteChallengeFix(req.game_id, req.challenge_id),
    onSuccess: () => {
      toastSuccess(t("general.actions.delete.status.success"));
      props.onSuccess?.();
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.delete.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function getChallengeKoh(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh`).json<KohStatus>();
}

export function useChallengeKoh({
  game_id,
  challenge_id,
  enabled,
  onError,
}: {
  game_id: () => number;
  challenge_id: () => number;
  enabled?: () => boolean;
  onError?: (err: Error) => boolean;
}) {
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "koh"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getChallengeKoh(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.koh.errors.fetch.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function updateChallengeKoh(game_id: number, challenge_id: number, config: KohConfig) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh`, {
      json: config,
    })
    .json<KohConfig>();
}

export function useUpdateChallengeKohMutation(
  props: { onSuccess?: (config: KohConfig) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; config: KohConfig }) =>
      updateChallengeKoh(req.game_id, req.challenge_id, req.config),
    onSuccess: (data: KohConfig) => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function deleteChallengeKoh(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh`).json<void>());
}

export function useDeleteChallengeKohMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void }) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => deleteChallengeKoh(req.game_id, req.challenge_id),
    onSuccess: () => {
      toastSuccess(t("general.actions.delete.status.success"));
      props.onSuccess?.();
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.delete.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function startKohHill(game_id: number, challenge_id: number) {
  return await safeJson(api.post(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh/hill`).json<void>());
}

export function useStartKohHillMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void } = {}) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => startKohHill(req.game_id, req.challenge_id),
    onSuccess: () => props.onSuccess?.(),
    onError: (err: Error) => {
      handleHttpError(err, t("challenge.koh.errors.start.title"));
      props.onError?.(err);
    },
  }));
}

export async function stopKohHill(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh/hill`).json<void>());
}

export function useStopKohHillMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void } = {}) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => stopKohHill(req.game_id, req.challenge_id),
    onSuccess: () => props.onSuccess?.(),
    onError: (err: Error) => {
      handleHttpError(err, t("challenge.koh.errors.stop.title"));
      props.onError?.(err);
    },
  }));
}

export async function checkKohOnce(game_id: number, challenge_id: number) {
  return await safeJson(api.post(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh/check`).json<void>());
}

export function useCheckKohOnceMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void } = {}) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => checkKohOnce(req.game_id, req.challenge_id),
    onSuccess: () => props.onSuccess?.(),
    onError: (err: Error) => {
      handleHttpError(err, t("challenge.koh.errors.check.title"));
      props.onError?.(err);
    },
  }));
}

export async function getKohEvents(game_id: number, challenge_id: number, limit?: number) {
  return await api
    .get(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh/event`, {
      searchParams: JSON.parse(JSON.stringify({ limit })),
    })
    .json<KohEvent[]>();
}

export function useKohEvents({
  game_id,
  challenge_id,
  limit,
  enabled,
  onError,
}: {
  game_id: () => number;
  challenge_id: () => number;
  limit?: () => number;
  enabled?: () => boolean;
  onError?: (err: Error) => boolean;
}) {
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "koh", "events", limit?.() ?? 50]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getKohEvents(game_id(), challenge_id(), limit?.() ?? 50),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.koh.errors.fetchEvents.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function getKohScoreboard(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/koh/scoreboard`).json<KohScore[]>();
}

export function useKohScoreboard({
  game_id,
  challenge_id,
  enabled,
  onError,
}: {
  game_id: () => number;
  challenge_id: () => number;
  enabled?: () => boolean;
  onError?: (err: Error) => boolean;
}) {
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "koh", "scoreboard"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getKohScoreboard(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.koh.errors.fetchScoreboard.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function submitFix(game_id: number, challenge_id: number, file: File) {
  const formData = new FormData();
  formData.append(file.name, file);
  return await api
    .post(`${api_root}/game/${game_id}/challenge/${challenge_id}/fix/submit`, {
      body: formData,
    })
    .json<Submission>();
}

export function useSubmitFixMutation(
  props: { onSuccess?: (submission: Submission) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; file: File }) =>
      submitFix(req.game_id, req.challenge_id, req.file),
    onSuccess: (data: Submission) => props.onSuccess?.(data),
    onError: (err: Error) => {
      handleHttpError(err, t("challenge.fix.errors.submit.title"));
      props.onError?.(err);
    },
  }));
}
