export interface Platform {
  name: string
  footer_info: string
  footer_url: string
  subject_info: string
  subject_url: string
  record: string | null
  hide_maker: boolean
}

export enum Validator {
  None = 0,
  Image = 1,
  Pow = 2,
  RecaptchaV3 = 3,
  HCaptcha = 4,
}

export interface Captcha {
  enabled: boolean
  difficulty: number
  validator: Validator
}

export interface Auth {
  signing_key: string
  buffer_time: number
  expires_time: number
}

export interface Email {
  enabled: boolean
  host: string
  port: number
  sender: string
  username: string
  password: string
  tls: 'none' | 'tls' | 'starttls'
  reset_password_email_body: string
  reset_password_email_subject: string
  verify_email_body: string
  verify_email_subject: string
}

export interface Media {
  anti_theft: boolean
  limit: number
}

export interface Pusher {
  enabled: boolean
  token: string
}

export interface Config {
  id?: number
  platform?: Platform
  captcha?: Captcha
  auth?: Auth
  email?: Email
  media?: Media
  pusher?: Pusher
}
