import type { Captcha } from '$lib/models/captcha'
import { api, api_root } from '.'

export interface LoginRequest {
  account: string
  password: string
  captcha_id: string
  captcha_answer: string
}

export async function login(request: LoginRequest) {
  return api.post(`${api_root}/account/login`, request)
}

export interface RegisterRequest {
  name: string
  email: string
  password: string
  captcha_id: string
  captcha_answer: string
}

export async function register(request: RegisterRequest) {
  return api.post(`${api_root}/account/register`, request)
}

export async function logout() {
  return api.post(`${api_root}/account/logout`)
}

export async function getCaptcha() {
  return (await api.get(`${api_root}/account/captcha`)).data as Captcha
}
