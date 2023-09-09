import type { Config } from '$lib/models/config'
import { api, api_root } from '.'

export async function getPlatformInfo() {
  return await api.GET(`${api_root}/platform`)
}

export async function testToken(token: string) {
  return await api.HEAD(`${api_root}/platform/config`, undefined, { headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' } })
}

export async function setPlatformConfig(config: Config, token: string) {
  return await api.POST(`${api_root}/platform/config`, config, { headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' } })
}
