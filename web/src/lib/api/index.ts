import { get } from 'svelte/store'
import { user, userExtractToken, userReset } from '$lib/stores/user'
import type { AxiosError, AxiosRequestHeaders, AxiosResponse } from 'axios'
import axios from 'axios'

const api = axios.create()

export const api_root = import.meta.env.VITE_API_ROOT

interface ExtractNewTokenHeaders {
  'set-token': string

  [key: string]: string
}

api.interceptors.request.use(
  (config) => {
    const token = get(user).token
    if (token) {
      if (config.headers) {
        config.headers['Authorization'] = token ? `Bearer ${token}` : ''
      } else {
        config.headers = { Authorization: token ? `Bearer ${token}` : '' } as unknown as AxiosRequestHeaders
      }
    }
    return config
  },
  (error: unknown) => {
    return Promise.reject(error)
  },
)

api.interceptors.response.use((response: AxiosResponse): AxiosResponse => {
  const headers: ExtractNewTokenHeaders =
    response.headers as ExtractNewTokenHeaders
  if (headers['set-token']) {
    userExtractToken(headers['set-token'])
  }
  return response
},
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      userReset()
    }
    return Promise.reject(error)
  },
)

export { api }
