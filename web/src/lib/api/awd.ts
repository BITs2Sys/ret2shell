// BITs2CTF fork: AWD + AWDP per-challenge API hooks. Fork-owned so upstream API
// changes don't conflict; mirrors the koh/isw hook shapes in ./challenge.ts + ./isw.ts.
import type { AwdConfig, AwdpConfig, AwdpStatus, AwdStatus } from "@models/awd";
import { t } from "@storage/theme";
import { useMutation, useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import api, { api_root, handleHttpError, inflyClient, safeJson, toastSuccess } from ".";

// ===========================================================================
// AWD
// ===========================================================================
export async function getChallengeAwd(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd`).json<AwdStatus>();
}

export function useChallengeAwd({
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
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "awd"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getChallengeAwd(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.awd.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function getAwdScoreboard(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd/scoreboard`).json();
}

export async function updateChallengeAwd(game_id: number, challenge_id: number, config: AwdConfig) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd`, { json: config })
    .json<AwdConfig>();
}

export function useUpdateChallengeAwdMutation(
  props: { onSuccess?: (config: AwdConfig) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; config: AwdConfig }) =>
      updateChallengeAwd(req.game_id, req.challenge_id, req.config),
    onSuccess: (data: AwdConfig) => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function deleteChallengeAwd(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd`).json<void>());
}

export function useDeleteChallengeAwdMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void }) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => deleteChallengeAwd(req.game_id, req.challenge_id),
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

// Admin: (re)provision one machine per team.
export async function provisionAwd(game_id: number, challenge_id: number) {
  return await api
    .post(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd/provision`)
    .json<{ created: number }>();
}

export function useProvisionAwdMutation(props: { onSuccess?: (r: { created: number }) => void } = {}) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => provisionAwd(req.game_id, req.challenge_id),
    onSuccess: (data: { created: number }) => {
      toastSuccess(t("challenge.awd.provisioned", { count: data.created }));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => handleHttpError(err, t("challenge.awd.title")),
  }));
}

// Admin: tear down every machine for this challenge.
export async function teardownAwd(game_id: number, challenge_id: number) {
  return await safeJson(api.post(`${api_root}/game/${game_id}/challenge/${challenge_id}/awd/teardown`).json<void>());
}

export function useTeardownAwdMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => teardownAwd(req.game_id, req.challenge_id),
    onSuccess: () => {
      toastSuccess(t("challenge.awd.tornDown"));
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("challenge.awd.title")),
  }));
}

// ===========================================================================
// AWDP
// ===========================================================================
export async function getChallengeAwdp(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/awdp`).json<AwdpStatus>();
}

export function useChallengeAwdp({
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
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "awdp"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getChallengeAwdp(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.awdp.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function getAwdpScoreboard(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/awdp/scoreboard`).json();
}

export async function updateChallengeAwdp(game_id: number, challenge_id: number, config: AwdpConfig) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/awdp`, { json: config })
    .json<AwdpConfig>();
}

export function useUpdateChallengeAwdpMutation(
  props: { onSuccess?: (config: AwdpConfig) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; config: AwdpConfig }) =>
      updateChallengeAwdp(req.game_id, req.challenge_id, req.config),
    onSuccess: (data: AwdpConfig) => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function deleteChallengeAwdp(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/awdp`).json<void>());
}

export function useDeleteChallengeAwdpMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void }) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => deleteChallengeAwdp(req.game_id, req.challenge_id),
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
