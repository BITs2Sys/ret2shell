import type { Captcha } from '$lib/models/captcha'
import type { User } from '$lib/models/user'
import { api, api_root } from '..'

export interface LoginRequest {
  account: string
  password: string
  captcha_id: string
  captcha_answer: string
}

export async function login(request: LoginRequest) {
  return api.post(`${api_root}/v1/account/login`, request)
}

export interface RegisterRequest {
  name: string
  email: string
  password: string
  captcha_id: string
  captcha_answer: string
}

export async function register(request: RegisterRequest) {
  return await api.post(`${api_root}/v1/account/register`, request)
}

export async function logout() {
  return await api.post(`${api_root}/v1/account/logout`)
}

export async function getCaptcha() {
  return (await api.get(`${api_root}/v1/account/captcha`)).data as Captcha
}

export async function updateSelfSetting(data: User) {
  return await api.patch(`${api_root}/v1/account/self`, data)
}

export interface ChangeUserPasswordRequest {
  old_password: string
  new_password: string
  captcha_id: string
  captcha_answer: string
}

export async function changeUserPassword(request: ChangeUserPasswordRequest) {
  return await api.patch(`${api_root}/v1/account/change-password`, request)
}

export async function resendEmailVerification() {
  return api.post(`${api_root}/v1/account/send-verification-email`)
}

export interface VerifyEmailRequest {
  email: string
  token: string
}

export async function verifyEmail(request: VerifyEmailRequest) {
  return api.post(`${api_root}/v1/account/verify-email`, request)
}

export interface SendResetEmailRequest {
  email: string
  captcha_id: string
  captcha_answer: string
}

export async function sendResetEmail(request: SendResetEmailRequest) {
  return api.post(`${api_root}/v1/account/send-reset-email`, request)
}

export interface ResetPasswordRequest {
  email: string
  password: string
  token: string
  captcha_id: string
  captcha_answer: string
}

export async function resetPassword(request: ResetPasswordRequest) {
  return api.post(`${api_root}/v1/account/reset-password`, request)
}

export async function deactivateAccount() {
  return api.post(`${api_root}/v1/account/delete`)
}
