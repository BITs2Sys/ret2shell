<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { platform } from '$lib/stores/platform'

  $platform.see_magic_category = true
</script>

<slot />

<RxLink
  square
  class={`fixed top-20 right-4 ${$page.url.pathname === '/magic' && 'hidden'}`}
  size="lg"
  href="/magic"
  exactlyMatched
>
  <span class="icon-[fluent--apps-list-20-regular] w-6 h-6"></span>
</RxLink>
