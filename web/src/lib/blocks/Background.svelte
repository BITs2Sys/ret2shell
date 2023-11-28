<script>
  import BackgroundLeft from '$lib/assets/background-left.svelte'
  import BackgroundRight from '$lib/assets/background-right.svelte'
  import BgBlur from '$lib/assets/imgs/bg-blur.webp'

  export let showBlurBg = true
</script>

{#if showBlurBg}
  <div class="fixed -left-1 -right-1 -top-1 -bottom-1 -z-50">
    <img
      src={BgBlur}
      alt=""
      class="w-full h-full object-fill"
      on:contextmenu={() => {
        return false
      }}
    />
    <div class="fixed left-0 right-0 top-0 bottom-0 bg-base-100/80 backdrop-blur"></div>
  </div>
{/if}
<BackgroundLeft />
<BackgroundRight />
