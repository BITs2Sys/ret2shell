import { user } from '$lib/stores/user'
import { error } from '@sveltejs/kit'
import { get } from 'svelte/store'

export function load() {
  if (!get(user).isLoggedIn) {
    throw error(401, 'Unauthorized')
  } else if (get(user).level < 2) {
    throw error(403, 'Forbidden')
  }
}
