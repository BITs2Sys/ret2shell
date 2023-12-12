import type { ClusterInfo, Config, Platform, PlatformStat } from '$lib/models/config'
import { api, api_root } from '..'

export async function getPlatformInfo() {
  return (await api.get(`${api_root}/v1/platform`)).data as Platform
}

export async function getPlatformVersion() {
  return (await api.get(`${api_root}/v1/platform/version`)).data as string
}

export async function testToken(token: string) {
  return await api.head(`${api_root}/v1/platform/config`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export async function getPlatformConfig() {
  return (await api.get(`${api_root}/v1/platform/config`)).data as Config
}

export async function setPlatformConfig(config: Config, token: string) {
  return await api.post(`${api_root}/v1/platform/config`, config, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export async function getPlatformStat() {
  return (await api.get(`${api_root}/v1/platform/stat`)).data as PlatformStat
}

export async function getPlatformClusterInfo() {
  return (await api.get(`${api_root}/v1/platform/cluster`)).data as ClusterInfo
}

export async function getPlatformLicense() {
  return (await api.get(`${api_root}/v1/platform/license`)).data as {
    issuer: string
    website: string
    date: string
  }
}
