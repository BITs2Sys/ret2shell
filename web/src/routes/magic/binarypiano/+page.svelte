<script lang="ts">
  import { Howl } from 'howler'
  import S00 from '$lib/assets/piano/G3.mp3'
  import S01 from '$lib/assets/piano/F3.mp3'
  import S02 from '$lib/assets/piano/E3.mp3'
  import S03 from '$lib/assets/piano/D3.mp3'
  import S04 from '$lib/assets/piano/C3.mp3'
  import S05 from '$lib/assets/piano/E5.mp3'
  import S06 from '$lib/assets/piano/D5.mp3'
  import S07 from '$lib/assets/piano/C5.mp3'
  import S08 from '$lib/assets/piano/A4.mp3'
  import S09 from '$lib/assets/piano/Ab4.mp3'
  import { fade } from 'svelte/transition'
  import { onDestroy } from 'svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'

  // sound array
  const soundsStr = ['S00', 'S01', 'S02', 'S03', 'S04', 'S05', 'S06', 'S07', 'S08', 'S09']
  let soundMap = new Map()

  let playing = false
  let num = 0
  let binaryNum = num.toString(2).padStart(10, '0').split('').reverse().slice(0, 10).reverse()
  let prevBinaryNum = num.toString(2).padStart(10, '0').split('')

  const timer = setInterval(() => {
    if (playing) num++
    binaryNum = num.toString(2).padStart(10, '0').split('').reverse().slice(0, 10).reverse()
    let played = false
    binaryNum.forEach((item, index) => {
      if (item !== prevBinaryNum[index]) {
        if (!played) {
          soundMap.get(soundsStr[index])?.play()
          played = true
        }
        prevBinaryNum[index] = item
      }
    })
  }, 200)
  onDestroy(() => {
    clearInterval(timer)
  })

  function startPlay() {
    if (soundMap.size === 0) {
      soundMap = new Map([
        ['S00', new Howl({ src: S00, volume: 1 })],
        ['S01', new Howl({ src: S01, volume: 1 })],
        ['S02', new Howl({ src: S02, volume: 1 })],
        ['S03', new Howl({ src: S03, volume: 1 })],
        ['S04', new Howl({ src: S04, volume: 1 })],
        ['S05', new Howl({ src: S05, volume: 1 })],
        ['S06', new Howl({ src: S06, volume: 0.8 })],
        ['S07', new Howl({ src: S07, volume: 0.6 })],
        ['S08', new Howl({ src: S08, volume: 0.8 })],
        ['S09', new Howl({ src: S09, volume: 0.3 })],
      ])
    }
    playing = !playing
  }
</script>

<svelte:head><title>{$i18n.t('surprise.binarypiano.title')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-col items-center justify-center space-y-16">
  <h1 class="text-3xl font-bold">A Music Made By Binary Numbers</h1>
  <div class="flex flex-row space-x-4">
    {#each binaryNum as item}
      <div
        class="w-16 h-16 flex items-center justify-center rounded-md bg-base-content/5 backdrop-blur text-2xl font-bold"
      >
        <span class="text-info">{item}</span>
        <div class="absolute w-full h-full top-0 left-0 flex flex-col items-center justify-center">
          {#key item}
            <span in:fade={{ duration: 300 }}>{item}</span>
          {/key}
        </div>
      </div>
    {/each}
    <RxButton class="w-16 h-16" on:click={startPlay}>
      {#if playing}
        <span class="icon-[fluent--pause-24-regular] w-5 h-5"></span>
      {:else}
        <span class="icon-[fluent--play-24-regular] w-5 h-5"></span>
      {/if}
    </RxButton>
  </div>
</div>
