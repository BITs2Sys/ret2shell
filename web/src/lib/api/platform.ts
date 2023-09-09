import { api, api_root } from '.'

export async function getPlatformInfo() {
  return await api.GET(`${api_root}/platform`)
}

export async function testToken(token: string) {
  return await api.HEAD(`${api_root}/platform/config`, undefined, { headers: { Authorization: `Bearer ${token}` } })
}
