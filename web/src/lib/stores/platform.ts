import { writable } from 'svelte/store'
import { browser } from '$app/environment'

class PlatformStore {
  name: string
  subject_info: string
  subject_url: string
  footer_info: string
  footer_url: string
  accept_cookies: boolean
  // 备案
  record: string | null
  hide_maker: boolean

  constructor() {
    if (browser) {
      const stored = localStorage.getItem('platform')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.name = parsed.name
        this.subject_info = parsed.subject_info
        this.subject_url = parsed.subject_url
        this.footer_info = parsed.footer_info
        this.footer_url = parsed.footer_url
        this.accept_cookies = parsed.accept_cookies
        this.record = parsed.record
        this.hide_maker = parsed.hide_maker
        return
      }
    }
    this.name = 'Ret 2 Shell'
    this.subject_info = 'Fighting for all the beauty in the world'
    this.subject_url = 'https://www.woooo.tech'
    this.footer_info = 'Wootec Inc.'
    this.footer_url = 'https://www.woooo.tech'
    this.accept_cookies = false
    this.record = null
    this.hide_maker = false
  }
}

export const platform = writable(new PlatformStore())

platform.subscribe((value) => {
  if (browser) localStorage.setItem('platform', JSON.stringify(value))
})
