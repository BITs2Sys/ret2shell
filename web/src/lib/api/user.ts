import type { Institute } from '$lib/models/institute'
import type { User } from '$lib/models/user'
import { api, api_root } from '.'

export async function getUserInfo(id: number) {
  return (await api.get(`${api_root}/user/${id}`)).data as User
}

export async function getUserList(page: number, per_page: number, order?: string, filter?: string) {
  let uri = `${api_root}/user?page=${page}&per_page=${per_page}`
  if (order && order.length > 0) uri += `&order=${order}`
  if (filter && filter.length > 0) uri += `&filter=${filter}`
  return (await api.get(uri)).data as {users: User[], total: number}
}

export async function getInstituteList() {
  return (await api.get(`${api_root}/user/institute`)).data as Institute[]
}
