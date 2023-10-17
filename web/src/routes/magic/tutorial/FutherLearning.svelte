<script lang="ts">
  import { fly } from 'svelte/transition'
  import { step } from './store'
  import { i18n } from '$lib/i18n'
  import { onMount } from 'svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import Barometer from '$lib/assets/barometer.svelte'
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
</script>

{#if !goNext}
  <div class="flex-1 flex flex-col justify-center items-center space-y-24" out:fly={{ y: -32, duration: 500 }}>
    <h1 class="text-4xl font-bold" in:fly={{ y: 32, duration: 1000 }}>
      👴&nbsp;{$i18n.t('tutorial.futherLearningTitle')}
    </h1>
    <div class="w-full max-w-5xl flex flex-row items-center" in:fly={{ y: 32, duration: 1000, delay: 500 }}>
      <div class="flex-shrink-0">
        <Barometer width={256} height={256} />
      </div>
      <div class="prose max-w-full">
        <p class="text-lg indent-8">
          {$i18n.t('tutorial.futherLearning1')}
        </p>
        <ul>
          <li>
            <a href="https://ctf-wiki.org/">CTF Wiki</a>
          </li>
          <li>
            <a href="https://ctf.probius.xyz/">Hello CTF! (CN version only)</a>
          </li>
          <li>
            <span>...</span>
          </li>
        </ul>
      </div>
    </div>
    <div in:fly={{ y: 16, duration: 1000, delay: 1500 }}>
      <RxButton size="lg" on:click={next}>
        <span class="icon-[fluent--thumb-like-20-regular] w-5 h-5"></span>
        <span>{$i18n.t('tutorial.next')}</span>
      </RxButton>
    </div>
  </div>
{/if}
