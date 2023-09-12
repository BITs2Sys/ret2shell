import type { Wiki } from "$lib/models/wiki";
import { api, api_root } from ".";

export async function getWikiList (parent_id?: number) {
    let uri = `${api_root}/wiki`
    if (parent_id) {
        uri += `?parent_id=${parent_id}`
    }
    return (await api.get(uri)).data as Wiki[]
}

export async function getWiki (id: number) {
    return (await api.get(`${api_root}/wiki/${id}`)).data as Wiki
}

export async function getRelatedWikis (id: number) {
    return (await api.get(`${api_root}/wiki/${id}/related`)).data as Wiki[]
}
