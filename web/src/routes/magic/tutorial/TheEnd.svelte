<script lang="ts">
  import { goto } from '$app/navigation'
  import LogoAnimate from '$lib/assets/logo-animate.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import { fly } from 'svelte/transition'
  import { onMount } from 'svelte'
  let goNext = true

  onMount(() => {
    setTimeout(() => {
      goNext = false
    })
  })

  function next() {
    goto('/')
  }
</script>

{#if !goNext}
  <div class="flex-1 flex flex-col justify-center items-center space-y-8" out:fly={{ y: -32, duration: 500 }}>
    <h1 class="text-4xl flex flex-row items-center space-x-8 font-bold" in:fly={{ y: 32, duration: 2000 }}>
      <LogoAnimate width={80} height={80} />
      <span>THE END But...</span>
    </h1>
    <p class="text-lg w-full max-w-3xl indent-8" in:fly={{ y: 32, duration: 1000, delay: 500 }}>
      {$i18n.t('tutorial.doNotProfit')}
    </p>
    <p class="w-full max-w-3xl text-lg indent-8" in:fly={{ y: 32, duration: 1000, delay: 1000 }}>
      {$i18n.t('tutorial.theEnd')}
    </p>
    <div class="flex flex-col space-y-4 !mt-24" in:fly={{ y: 16, duration: 1000, delay: 2000 }}>
      <RxButton size="lg" on:click={next}>
        <span class="icon-[fluent--thumb-like-20-regular] w-5 h-5"></span>
        <span>{$i18n.t('tutorial.end')}</span>
      </RxButton>
    </div>
  </div>
{/if}
