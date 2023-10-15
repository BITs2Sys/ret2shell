import { writable } from 'svelte/store'
import { browser } from '$app/environment'

class PlatformStore {
  name: string
  subject_info: string
  subject_url: string
  footer_info: string
  footer_url: string
  accept_cookies: boolean
  see_custom_box: boolean
  see_magic_category: boolean
  // 备案
  record: string | null
  hide_maker: boolean

  constructor() {
    if (browser) {
      const stored = localStorage.getItem('platform')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.name = parsed.name ? parsed.name : 'Ret 2 Shell'
        this.subject_info = parsed.subject_info ? parsed.subject_info : 'Fighting for all the beauty in the world'
        this.subject_url = parsed.subject_url ? parsed.subject_url : 'https://www.woooo.tech'
        this.footer_info = parsed.footer_info ? parsed.footer_info : 'Wootec Inc.'
        this.footer_url = parsed.footer_url ? parsed.footer_url : 'https://www.woooo.tech'
        this.accept_cookies = parsed.accept_cookies ? true : false
        this.record = parsed.record ? parsed.record : ''
        this.hide_maker = parsed.hide_maker ? true : false
        this.see_custom_box = parsed.see_custom_box ? true : false
        this.see_magic_category = parsed.see_magic_category ? true : false
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
    this.see_custom_box = false
    this.see_magic_category = false
  }
}

export const platform = writable(new PlatformStore())

platform.subscribe((value) => {
  if (browser) localStorage.setItem('platform', JSON.stringify(value))
})
