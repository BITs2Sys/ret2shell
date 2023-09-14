<script lang="ts">
  import { goto } from '$app/navigation'
    import { page } from '$app/stores'
  import { user } from '$lib/stores/user'

  if (!$user.isLoggedIn) {
    goto(`/account/login?redirect=${$page.url.pathname}`)
  }
</script>

<slot />
