import { get, writable } from 'svelte/store'
import { browser } from '$app/environment'

class ThemeStore {
  colorScheme = 'dark'
  constructor() {
    if (browser) {
      const stored = localStorage.getItem('theme')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.colorScheme = parsed.colorScheme
        return
      }
      const systemPrefersTheme =
        window && window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
      this.colorScheme = systemPrefersTheme ? 'dark' : 'light'
    }
  }
}

export const theme = writable(new ThemeStore())

theme.subscribe((value) => {
  if (browser) {
    localStorage.setItem('theme', JSON.stringify(value))
  }
})

export function colorDefs() {
  if (get(theme).colorScheme === 'dark') {
    return {
      'base-content': 'hsl(145 0% 79%)',
      // border is base-content/5
      border: 'hsl(145 0% 15%)',
      primary: '#3399FF',
      secondary: '#60a5fa',
      accent: '#1FB2A6',
      neutral: '#202020',
      'base-100': '#121212',
      info: '#3399FF',
      success: '#36D399',
      warning: '#FBBD23',
      error: '#F83030',
    }
  } else {
    return {
      'base-content': 'hsl(146 0% 19%)',
      // border is base-content/5
      border: 'hsl(146 0% 89%)',
      primary: '#0078D6',
      secondary: '#60a5fa',
      accent: '#1FB2A6',
      neutral: '#F0F0F0',
      'base-100': '#FFFFFF',
      info: '#0078D6',
      success: '#36AA3A',
      warning: '#ca9f00',
      error: '#F83030',
    }
  }
}
