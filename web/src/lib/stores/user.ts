import { get, writable } from 'svelte/store'
import { browser } from '$app/environment'
import type { Permission, Token, User } from '$lib/models/user'
import { fromBase64 } from 'js-base64'
import { getUserInfo } from '$lib/api/v1/user'
import type { Instance } from '$lib/models/instance'

class UserStore {
  token = ''
  id = -1
  name = ''
  permissions: Permission[] = []
  isLoggedIn = false
  info: User | null = null
  runningInstance: Instance | null = null

  constructor() {
    if (browser) {
      const stored = localStorage.getItem('user')
      if (stored) {
        const parsed = JSON.parse(stored)
        this.token = parsed.token
        this.id = parsed.id
        this.name = parsed.name
        this.permissions = parsed.permissions
        this.isLoggedIn = parsed.isLoggedIn
        this.info = parsed.info
        return
      }
    }
  }
}

export const user = writable(new UserStore())

user.subscribe((value) => {
  if (browser) localStorage.setItem('user', JSON.stringify(value))
})

export function userReset() {
  user.update((value) => {
    value.token = ''
    value.id = -1
    value.name = ''
    value.permissions = []
    value.isLoggedIn = false
    value.info = null
    return value
  })
}

export function userExtractToken(token: string) {
  user.update((value) => {
    value.token = token
    const tokenRaw = fromBase64(token.split('.')[1])
    const tokenJson = JSON.parse(tokenRaw) as Token
    value.id = tokenJson.id
    value.name = tokenJson.name
    value.permissions = tokenJson.permissions
    value.isLoggedIn = true
    return value
  })
}

async function _fetchUserInfo() {
  try {
    const response = await getUserInfo(get(user).id)
    user.update((value) => {
      // console.log(response)
      value.info = response
      return value
    })
  } catch {
    return Promise.resolve()
  }
  return Promise.resolve()
}

export async function userInfo() {
  if (!get(user).info && get(user).isLoggedIn) {
    await _fetchUserInfo()
  }
  return get(user).info
}

export async function refreshUserInfo() {
  if (get(user).isLoggedIn) await _fetchUserInfo()
  else userReset()
}
