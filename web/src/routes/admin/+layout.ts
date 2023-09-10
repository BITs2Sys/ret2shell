import { Permission } from '$lib/models/user'
import { user } from '$lib/stores/user'
import { error } from '@sveltejs/kit'
import { get } from 'svelte/store'

export function load() {
  if (!get(user).isLoggedIn) {
    throw error(401, 'Unauthorized')
  } else if (get(user).permissions.every((p) => p < Permission.Publish)) {
    throw error(403, 'Forbidden')
  }
}
