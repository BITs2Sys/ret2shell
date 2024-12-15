import type { ConfigMapList, NodeList } from "kubernetes-types/core/v1";
import { DateTime } from "luxon";
import api, { api_root } from ".";

export async function getClusterConfig() {
  return await api.get(`${api_root}/cluster/config`).json<ConfigMapList>();
}

export async function getClusterNodes() {
  return await api.get(`${api_root}/cluster/node`).json<NodeList>();
}

export async function getCalmdownStatus() {
  const result = await api.get(`${api_root}/cluster/calmdown`).json<number | null>();
  if (result) {
    return DateTime.fromSeconds(result);
  }
  return null;
}
