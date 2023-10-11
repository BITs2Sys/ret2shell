import type { Instance } from '$lib/models/instance'
import { api, api_root } from '.'

export async function getSelfRunningInstance() {
  return (await api.get(`${api_root}/instance/self`)).data as Instance
}
