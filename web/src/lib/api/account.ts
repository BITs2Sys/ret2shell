import { api, api_root } from '.'

export interface LoginRequest {
  account: string
  password: string
  captcha_id: string
  captcha_answer: string
}

export async function login(request: LoginRequest) {
  return api.POST(`${api_root}/account/login`, { ...request })
}

export async function logout() {
  return api.POST(`${api_root}/account/logout`)
}

export async function getCaptcha() {
  return api.GET(`${api_root}/account/captcha`)
}
