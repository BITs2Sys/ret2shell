<script lang="ts">
  import { goto } from '$app/navigation'
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'

  if (!$user.isLoggedIn) {
    goto('/account/login', { replaceState: true }).then(() => {
      showMessage('warning', $i18n.t('permissions.beLoggedInToView'), 5000)
    })
  } else if (!$user.permissions.find((p) => p === Permission.Verified)) {
    goto('/account/profile', { replaceState: true }).then(() => {
      showMessage('warning', $i18n.t('permissions.beVerifiedToView'), 5000)
    })
  }
</script>

<slot />
