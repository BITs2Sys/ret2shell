import { writable } from 'svelte/store'
import { browser } from '$app/environment'

class PlatformStore {
  name: string
  subjectInfo: string
  subjectUrl: string
  footerInfo: string
  footerUrl: string
  acceptCookies: boolean
  // 备案
  record: string | null
  hideMaker: boolean

  constructor() {
    if (browser) {
      const stored = localStorage.getItem('platform')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.name = parsed.name
        this.subjectInfo = parsed.subjectInfo
        this.subjectUrl = parsed.subjectUrl
        this.footerInfo = parsed.footerInfo
        this.footerUrl = parsed.footerUrl
        this.acceptCookies = parsed.acceptCookies
        this.record = parsed.record
        this.hideMaker = parsed.hideMaker
        return
      }
    }
    this.name = 'Ret 2 Shell'
    this.subjectInfo = 'Fighting for all the beauty in the world'
    this.subjectUrl = 'https://www.woooo.tech'
    this.footerInfo = 'Wootec Inc.'
    this.footerUrl = 'https://www.woooo.tech'
    this.acceptCookies = false
    this.record = null
    this.hideMaker = false
  }
}

export const platform = writable(new PlatformStore())

platform.subscribe((value) => {
  if (browser) localStorage.setItem('platform', JSON.stringify(value))
})
