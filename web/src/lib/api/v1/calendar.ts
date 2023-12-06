import type { Calendar } from '$lib/models/calendar'
import { api, api_root } from '..'

export async function getCalendarList(startTime: number, endTime: number) {
  return (await api.get(`${api_root}/v1/calendar?start_time=${startTime}&end_time=${endTime}`)).data as Calendar[]
}

export async function getCalendar(id: number) {
  return (await api.get(`${api_root}/v1/calendar/${id}`)).data as Calendar
}

export async function createCalendar(calendar: Calendar) {
  return await api.post(`${api_root}/v1/calendar`, calendar)
}

export async function updateCalendar(id: number, calendar: Calendar) {
  return await api.patch(`${api_root}/v1/calendar/${id}`, calendar)
}

export async function deleteCalendar(id: number) {
  return await api.delete(`${api_root}/v1/calendar/${id}`)
}
