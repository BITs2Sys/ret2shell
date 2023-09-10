import type { Calendar } from '$lib/models/calendar'
import { api, api_root } from '.'

export async function getCalendarList(startTime: number, endTime: number) {
    return (await api.get(`${api_root}/calendar?start_time=${startTime}&end_time=${endTime}`)).data as Calendar[]
}
