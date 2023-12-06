import type { Wiki } from '$lib/models/wiki'
import { api, api_root } from '..'

export async function getWikiList(parent_id?: number) {
  let uri = `${api_root}/v1/wiki`
  if (parent_id) {
    uri += `?parent_id=${parent_id}`
  }
  return (await api.get(uri)).data as Wiki[]
}

export async function getWiki(id: number) {
  return (await api.get(`${api_root}/v1/wiki/${id}`)).data as Wiki
}

export async function getRelatedWikis(id: number) {
  return (await api.get(`${api_root}/v1/wiki/${id}/related`)).data as Wiki[]
}

export async function createWiki(wiki: Wiki) {
  return (await api.post(`${api_root}/v1/wiki`, wiki)).data as Wiki
}

export async function editWiki(id: number, wiki: Wiki) {
  return (await api.patch(`${api_root}/v1/wiki/${id}`, wiki)).data as Wiki
}

export async function deleteWiki(id: number) {
  return await api.delete(`${api_root}/v1/wiki/${id}`)
}
