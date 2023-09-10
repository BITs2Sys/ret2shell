import type { Config, Platform } from '$lib/models/config'
import { api, api_root } from '.'

export async function getPlatformInfo () {
  return (await api.get(`${api_root}/platform`)).data as Platform
}

export async function testToken (token: string) {
  return await api.head(`${api_root}/platform/config`, {
    headers: { Authorization: `Bearer ${token}` },
  })
}

export async function setPlatformConfig (config: Config, token: string) {
  return await api.post(`${api_root}/platform/config`, config, {
    headers: { Authorization: `Bearer ${token}` },
  })
}
