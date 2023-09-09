import type { Config } from '$lib/models/config'
import { writable } from 'svelte/store'

class InitConfigStore {
  config: Config
  token: string
  constructor() {
    this.config = {}
    this.token = ''
  }
}

export const initConfig = writable(new InitConfigStore())
