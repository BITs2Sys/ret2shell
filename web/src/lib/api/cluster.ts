import type { DiagnosticMarker } from "@widgets/editor";
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

export async function updateGlobalTrafficScript(traffic: string) {
  return await api.patch(`${api_root}/cluster/traffic`, { json: { traffic } }).json<{
    lint: DiagnosticMarker[] | null;
  }>();
}

export async function deleteGlobalTrafficScript() {
  return await api.delete(`${api_root}/cluster/traffic`).json<void>();
}

export async function updateDefaultNodeSelector(node_selector: string) {
  return await api.patch(`${api_root}/cluster/node-selector`, { json: { node_selector } }).json<void>();
}

export async function deleteDefaultNodeSelector() {
  return await api.delete(`${api_root}/cluster/node-selector`).json<void>();
}
