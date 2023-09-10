import { goto } from "$app/navigation"
import { user } from "$lib/stores/user"
import { get } from "svelte/store"

export function load () {
    if (!get(user).isLoggedIn) {
        goto("/account/login")
    }
}
