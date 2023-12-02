import { goto } from '$app/navigation'
import { page } from '$app/stores'
import { user } from '$lib/stores/user'
import { get } from 'svelte/store'

if (!get(user).isLoggedIn) {
  goto(`/account/login?redirect=${get(page).url.pathname}`, { replaceState: true })
}
