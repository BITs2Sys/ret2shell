import type { Announcement } from '$lib/models/announcement'
import { api, api_root } from '..'

export interface AnnouncementListResponse {
  announcements: Announcement[]
  total: number
}

export async function getAnnouncementList(page: number, per_page: number) {
  const response = await api.get(`${api_root}/v1/announcement?page=${page}&per_page=${per_page}`)
  return Promise.resolve(response.data as AnnouncementListResponse)
}

export async function getAnnouncement(id: number) {
  const response = await api.get(`${api_root}/v1/announcement/${id}`)
  return Promise.resolve(response.data as Announcement)
}

export async function createAnnouncement(announcement: Announcement) {
  return await api.post(`${api_root}/v1/announcement`, announcement)
}

export async function updateAnnouncement(id: number, announcement: Announcement) {
  return await api.patch(`${api_root}/v1/announcement/${id}`, announcement)
}

export async function deleteAnnouncement(id: number) {
  return await api.delete(`${api_root}/v1/announcement/${id}`)
}
