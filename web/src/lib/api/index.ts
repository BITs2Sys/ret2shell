import { get } from 'svelte/store'
import { user } from '$lib/stores/user'

export const api_root = '/api'

export class api {
  static async fetchWithToken(
    method: string | undefined,
    input: RequestInfo | URL,
    body?: Record<string, string>,
    init?: RequestInit | undefined
  ) {
    const token = get(user).token
    let headers: HeadersInit = {}
    if (token && token.length > 0) {
      headers = { Authentication: `Bearer ${token}`, 'Content-Type': 'application/json', ...init?.headers }
    } else {
      headers = { 'Content-Type': 'application/json', ...init?.headers }
    }
    let resp = await fetch(input, { method, headers, body: JSON.stringify(body), ...init })
    if (resp.headers.get('Set-Token')) {
      user.update((value) => {
        value.token = resp.headers.get('Set-Token')!
        return value
      })
    }
    return resp
  }

  static async GET(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('GET', input, body, init)
  }

  static async POST(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('POST', input, body, init)
  }

  static async PATCH(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('PATCH', input, body, init)
  }

  static async PUT(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('PUT', input, body, init)
  }

  static async DELETE(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('DELETE', input, body, init)
  }

  static async OPTIONS(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('OPTIONS', input, body, init)
  }

  static async HEAD(input: RequestInfo | URL, body?: Record<string, string>, init?: RequestInit) {
    return this.fetchWithToken('HEAD', input, body, init)
  }
}
