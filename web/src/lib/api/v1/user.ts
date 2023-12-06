import type { Institute } from '$lib/models/institute'
import type { IpAddress } from '$lib/models/ip'
import type { TeamWithGameName } from '$lib/models/team'
import type { User } from '$lib/models/user'
import { api, api_root } from '..'

export async function getUserInfo(id: number) {
  return (await api.get(`${api_root}/v1/user/${id}`)).data as User
}

export async function getUserList(page: number, per_page: number, order?: string, filter?: string) {
  let uri = `${api_root}/v1/user?page=${page}&per_page=${per_page}`
  if (order && order.length > 0) uri += `&order=${order}`
  if (filter && filter.length > 0) uri += `&filter=${filter}`
  return (await api.get(uri)).data as { users: User[]; total: number }
}

export async function getInstituteList() {
  return (await api.get(`${api_root}/v1/user/institute`)).data as Institute[]
}

export async function getUserTeams(id: number) {
  return (await api.get(`${api_root}/v1/user/${id}/teams`)).data as TeamWithGameName[]
}

export async function getUserIpAddresses(id: number) {
  return (await api.get(`${api_root}/v1/user/${id}/ip`)).data as IpAddress[]
}

export async function updateUser(id: number, data: User) {
  return (await api.patch(`${api_root}/v1/user/${id}`, data)).data as User
}
