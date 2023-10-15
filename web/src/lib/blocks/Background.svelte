<script>
  import BackgroundLeft from '$lib/assets/background-left.svelte'
  import BackgroundRight from '$lib/assets/background-right.svelte'
  import BgBlur from '$lib/assets/bg-blur.webp'
</script>

<div class="fixed left-0 right-0 top-0 bottom-0 -z-50">
  <img src={BgBlur} alt="" class="w-full h-full object-fill" />
  <div class="fixed left-0 right-0 top-0 bottom-0 bg-base-100/90 backdrop-blur"></div>
</div>
<BackgroundLeft />
<BackgroundRight />
