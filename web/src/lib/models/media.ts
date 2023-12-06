import { api_root } from '$lib/api'

export interface Media {
  id: number
  name: string
  hash: string
  uploader_id: number
}

export function getMediaRelPath(media: Media) {
  const path = `${media.hash.slice(0, 2)}/${media.hash.slice(2, 4)}`
  return `${path}/${media.name}`
}

export function getMediaPath(media: Media) {
  return appendMediaPath(getMediaRelPath(media))
}

export function getThumbnailPath(media: Media) {
  return appendThumbnailPath(getMediaRelPath(media))
}

export function appendMediaPath(media: string) {
  return `${api_root}/v1/media/${media}`
}

export function appendThumbnailPath(media: string) {
  return `${api_root}/v1/media/thumbnails/${media}`
}
