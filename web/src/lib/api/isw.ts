// BITs2CTF fork: ISW per-challenge config API hooks. Fork-owned so upstream API
// changes don't conflict; mirrors the koh hook shape in ./challenge.ts.
import type { IswConfig } from "@models/isw";
import { t } from "@storage/theme";
import { useMutation, useQuery } from "@tanstack/solid-query";
import { createMemo } from "solid-js";
import api, { api_root, handleHttpError, inflyClient, safeJson, toastSuccess } from ".";

export async function getChallengeIsw(game_id: number, challenge_id: number) {
  return await api.get(`${api_root}/game/${game_id}/challenge/${challenge_id}/isw`).json<IswConfig | null>();
}

// Download my team's WireGuard config for its assigned range (plain text).
export async function getMyRangeVpn(game_id: number) {
  return await api.get(`${api_root}/game/${game_id}/range/vpn`).text();
}

export function useChallengeIsw({
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
  const keys = createMemo(() => ["game", game_id(), "challenge", challenge_id(), "isw"]);
  return useQuery(
    () => ({
      queryKey: keys(),
      queryFn: async () => await getChallengeIsw(game_id(), challenge_id()),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("challenge.isw.title"));
        return onError?.(err) ?? false;
      },
    }),
    () => inflyClient
  );
}

export async function updateChallengeIsw(game_id: number, challenge_id: number, config: IswConfig) {
  return await api
    .patch(`${api_root}/game/${game_id}/challenge/${challenge_id}/isw`, { json: config })
    .json<IswConfig>();
}

export function useUpdateChallengeIswMutation(
  props: { onSuccess?: (config: IswConfig) => void; onError?: (err: Error) => void } = {}
) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number; config: IswConfig }) =>
      updateChallengeIsw(req.game_id, req.challenge_id, req.config),
    onSuccess: (data: IswConfig) => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.(data);
    },
    onError: (err: Error) => {
      handleHttpError(err, t("general.actions.save.status.fail"));
      props.onError?.(err);
    },
  }));
}

export async function deleteChallengeIsw(game_id: number, challenge_id: number) {
  return await safeJson(api.delete(`${api_root}/game/${game_id}/challenge/${challenge_id}/isw`).json<void>());
}

export function useDeleteChallengeIswMutation(props: { onSuccess?: () => void; onError?: (err: Error) => void }) {
  return useMutation(() => ({
    mutationFn: (req: { game_id: number; challenge_id: number }) => deleteChallengeIsw(req.game_id, req.challenge_id),
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
