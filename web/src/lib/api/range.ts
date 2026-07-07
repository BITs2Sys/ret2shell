// BITs2CTF fork: ISW range-mode admin API (DevOps-gated /range endpoints).
import type { ArmReport, IswHost, IswRange, IswRangeTemplate, RangeDetail, RangeHealth } from "@models/range";
import { t } from "@storage/theme";
import { useMutation, useQuery } from "@tanstack/solid-query";
import api, { api_root, handleHttpError, inflyClient, safeJson, toastSuccess } from ".";

const root = `${api_root}/range`;

// ---- hosts ----
export function useIswHosts() {
  return useQuery(
    () => ({
      queryKey: ["range", "host"],
      queryFn: async () => await api.get(`${root}/host`).json<IswHost[]>(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("range.host.title"));
        return false;
      },
    }),
    () => inflyClient
  );
}

export function useCreateHostMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (body: { name: string; address: string; api_port: number; os: string; fingerprint?: string | null }) =>
      api.post(`${root}/host`, { json: body }).json<IswHost>(),
    onSuccess: () => {
      toastSuccess(t("general.actions.save.status.success"));
      inflyClient.invalidateQueries({ queryKey: ["range", "host"] });
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.save.status.fail")),
  }));
}

export function useDeleteHostMutation() {
  return useMutation(() => ({
    mutationFn: (id: number) => safeJson(api.delete(`${root}/host/${id}`).json<void>()),
    onSuccess: () => {
      toastSuccess(t("general.actions.delete.status.success"));
      inflyClient.invalidateQueries({ queryKey: ["range", "host"] });
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.delete.status.fail")),
  }));
}

export function useProbeHostMutation(props: { onSuccess?: (h: RangeHealth) => void } = {}) {
  return useMutation(() => ({
    mutationFn: (id: number) => api.get(`${root}/host/${id}/health`).json<RangeHealth>(),
    onSuccess: (data: RangeHealth) => {
      toastSuccess(t("range.host.online"));
      inflyClient.invalidateQueries({ queryKey: ["range", "host"] });
      props.onSuccess?.(data);
    },
    onError: (err: Error) => handleHttpError(err, t("range.host.probeFail")),
  }));
}

// ---- templates ----
export function useIswTemplates(gameId: () => number) {
  return useQuery(
    () => ({
      queryKey: ["range", "template", gameId()],
      queryFn: async () =>
        await api.get(`${root}/template`, { searchParams: { game_id: gameId() } }).json<IswRangeTemplate[]>(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("range.template.title"));
        return false;
      },
    }),
    () => inflyClient
  );
}

export function useCreateTemplateMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (body: { game_id: number; name: string; brief: string; topology: unknown }) =>
      api.post(`${root}/template`, { json: body }).json<IswRangeTemplate>(),
    onSuccess: (data: IswRangeTemplate) => {
      toastSuccess(t("general.actions.save.status.success"));
      inflyClient.invalidateQueries({ queryKey: ["range", "template", data.game_id] });
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.save.status.fail")),
  }));
}

// ---- ranges ----
export function useIswRange(rangeId: () => number, enabled?: () => boolean) {
  return useQuery(
    () => ({
      queryKey: ["range", "instance", rangeId()],
      queryFn: async () => await api.get(`${root}/instance/${rangeId()}`).json<RangeDetail>(),
      enabled: enabled?.(),
      throwOnError: (err: Error) => {
        handleHttpError(err, t("range.instance.title"));
        return false;
      },
    }),
    () => inflyClient
  );
}

export function useCreateRangeMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (body: { template_id: number; host_id: number; group_index: number; name: string }) =>
      api.post(`${root}/instance`, { json: body }).json<IswRange>(),
    onSuccess: () => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.save.status.fail")),
  }));
}

function rangeAction(action: "arm" | "snapshot" | "reset", labelKey: string) {
  return () =>
    useMutation(() => ({
      mutationFn: async (id: number) => {
        const res = api.post(`${root}/instance/${id}/${action}`);
        return action === "snapshot"
          ? ((await safeJson(res.json<void>())) as unknown as ArmReport | null)
          : await res.json<ArmReport>();
      },
      onSuccess: () => {
        toastSuccess(t(labelKey));
        inflyClient.invalidateQueries({ queryKey: ["range", "instance"] });
      },
      onError: (err: Error) => handleHttpError(err, t(labelKey)),
    }));
}

export const useArmRangeMutation = rangeAction("arm", "range.actions.arm");
export const useSnapshotRangeMutation = rangeAction("snapshot", "range.actions.snapshot");
export const useResetRangeMutation = rangeAction("reset", "range.actions.reset");

export function useDeleteRangeMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (id: number) => safeJson(api.delete(`${root}/instance/${id}`).json<void>()),
    onSuccess: () => {
      toastSuccess(t("general.actions.delete.status.success"));
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.delete.status.fail")),
  }));
}

// ---- assignments ----
export function useCreateAssignmentMutation(props: { onSuccess?: () => void } = {}) {
  return useMutation(() => ({
    mutationFn: (body: { game_id: number; range_id: number; team_id: number }) =>
      api.post(`${root}/assignment`, { json: body }).json<unknown>(),
    onSuccess: () => {
      toastSuccess(t("general.actions.save.status.success"));
      props.onSuccess?.();
    },
    onError: (err: Error) => handleHttpError(err, t("general.actions.save.status.fail")),
  }));
}
