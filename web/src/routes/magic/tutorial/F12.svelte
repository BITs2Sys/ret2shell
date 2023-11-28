<script lang="ts">
  import { fly } from 'svelte/transition'
  import { step } from './store'
  import LocationAnimate from '$lib/assets/animates/location-animate.svelte'
  import { i18n } from '$lib/i18n'
  import { onMount } from 'svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  let goNext = true

  onMount(() => {
    setTimeout(() => {
      goNext = false
    })
  })

  function next() {
    goNext = true
    setTimeout(() => {
      $step = $step + 1
    }, 500)
  }

  let flag = ''
  let expected = 'flag{hey_you_find_me!!!}'
</script>

{#if !goNext}
  <div class="flex-1 flex flex-col justify-center items-center space-y-24" out:fly={{ y: -32, duration: 500 }}>
    <h1 class="text-4xl font-bold" in:fly={{ y: 32, duration: 1000 }}>
      🤔&nbsp;{$i18n.t('tutorial.whatIsF12Title')}
    </h1>
    <div class="w-full max-w-5xl flex flex-row items-center" in:fly={{ y: 32, duration: 1000, delay: 500 }}>
      <div class="prose max-w-full">
        <div class="flex flex-row items-center">
          <div class="flex-shrink-0 pr-12 text-primary">
            <LocationAnimate width={128} height={128} />
            <p class="hidden">{'flag{hey_you_find_me!!!}'}</p>
          </div>
          <p class="text-lg indent-8">
            {$i18n.t('tutorial.whatIsF12')}
          </p>
        </div>
        <div class="divider"></div>
        <p class="text-lg indent-8">
          {$i18n.t('tutorial.tryF12Flag')}
        </p>
        <p class="text-lg indent-8">{$i18n.t('tutorial.inputFlagAndNext')}</p>
      </div>
    </div>
    <div class="w-full max-w-5xl flex flex-row" in:fly={{ y: 16, duration: 1000, delay: 1500 }}>
      <RxInput
        class="flex-1"
        icon="icon-[fluent--flag-20-regular]"
        bind:value={flag}
        placeholder={$i18n.t('tutorial.inputFlagAndNext')}
      >
        <RxButton class="join-item ml-0" on:click={next} disabled={flag !== expected}>
          <span class="icon-[fluent--thumb-like-20-regular] w-5 h-5"></span>
          <span>{$i18n.t('tutorial.next')}</span>
        </RxButton>
      </RxInput>
    </div>
  </div>
{/if}
