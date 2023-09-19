import { game } from '$lib/stores/game'
import { userInfo } from '$lib/stores/user'
import { get } from 'svelte/store'

export async function canTakePartInGame() {
  const gameStore = get(game)
  const user = await userInfo()
  if (gameStore.current && user) {
    if (!gameStore.current.host_as_game) {
      return false
    }
    if (new Date(gameStore.current.register_time * 1000) > new Date()) {
      return false
    }
    if (new Date(gameStore.current.start_time * 1000) < new Date() && !gameStore.current.can_register_after_started) {
      return false
    }
    if (new Date(gameStore.current.end_time * 1000) < new Date()) {
      return false
    }
    if (gameStore.current.institute_id !== null && gameStore.current.institute_id !== user.institute_id) {
      return false
    }
    return true
  }
  return false
}
