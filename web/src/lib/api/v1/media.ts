import type { Media } from '$lib/models/media'
import type { Writable } from 'svelte/store'
import { api, api_root } from '..'

export interface UploadMediaResponse {
  model: Media
  remaining: number
}

export async function uploadMedia(file: File, requireThumbnail: boolean, progress?: Writable<number | undefined>) {
  const formData = new FormData()
  formData.append('data', file)
  const resp = await api.post(`${api_root}/v1/media/upload?require_thumbnail=${requireThumbnail}`, formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    onUploadProgress: (e) => {
      if (progress) {
        progress.update((p) => {
          p = (e.progress || 1) * 100
          return p
        })
        // console.log("update progress to", progress.value)
      }
    },
  })
  return resp.data as UploadMediaResponse
}
