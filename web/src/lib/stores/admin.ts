import { i18n } from '$lib/i18n'
import type { Challenge } from '$lib/models/challenge'
import type { Game } from '$lib/models/game'
import { get, writable } from 'svelte/store'

interface RouteItem {
  name: string
  path: string
}

class AdminStore {
  game: Game | null
  challenge: Challenge | null
  route: RouteItem[]

  constructor() {
    this.game = null
    this.challenge = null
    this.route = [{ name: get(i18n).t('admin.title'), path: '/admin' }]
  }
}

export const admin = writable(new AdminStore())

export function refreshAdminRoute(path: string) {
  let routes = path.split('/').filter((x) => x !== '')
  let routeItems: RouteItem[] = []
  let routePath = ''
  for (const route in routes) {
    routePath += '/' + routes[route]
    routeItems.push({ name: get(i18n).t(`admin.routes.${routes[route]}`), path: routePath })
  }

  admin.update((a) => {
    a.route = routeItems
    return a
  })
}
