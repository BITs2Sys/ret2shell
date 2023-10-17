<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxPing from '$lib/components/RxPing.svelte'
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
  <div class="w-full h-full relative flex items-center justify-center">
    <RxPing level="info" />
    <span class="icon-[fluent--apps-list-20-regular] w-5 h-5"></span>
  </div>
</RxLink>
