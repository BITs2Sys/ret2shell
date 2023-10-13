<script lang="ts">
  import { fly } from 'svelte/transition'
  import { step } from './store'
  import { onMount } from 'svelte'
  import LogoAnimate from '$lib/assets/logo-animate.svelte'
  import ArrowUpAnimate from '$lib/assets/ArrowUpAnimate.svelte'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxButton from '$lib/components/RxButton.svelte'
  let goNext = true
  let routeItemIndex = 0

  onMount(() => {
    setTimeout(() => {
      goNext = false
    })
  })

  function next() {
    if (routeItemIndex < 4) {
      routeItemIndex++
    } else {
      goNext = true
      setTimeout(() => {
        $step++
      }, 500)
    }
  }
</script>

{#if !goNext}
  <div class="flex-1 flex flex-col justify-center items-center space-y-24 relative" out:fly={{ y: -32, duration: 500 }}>
    <div class="absolute h-40 top-0 left-0 w-full px-2 py-0 flex flex-row">
      <div class="btn text-base space-x-2 invisible">
        <span>
          <LogoAnimate width={28} height={28} />
        </span>
        <span>{$platform.name}</span>
      </div>
      <ul class="menu menu-horizontal space-x-2 m-0 p-0 px-6">
        <li class="relative">
          <div class="btn text-base space-x-2 invisible">
            <span class="icon-[fluent--book-number-16-regular] w-6 h-6" />
            {$i18n.t('wiki.title')}
          </div>
          {#if routeItemIndex === 1}
            <div
              class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <div>
                <ArrowUpAnimate width={200} height={200} />
              </div>
            </div>
            <div
              class="absolute top-40 left-0 w-96 p-6 rounded-box bg-neutral/80 backdrop-blur"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <p class="w-full text-lg indent-8">
                {$i18n.t('tutorial.platformInfoWiki')}
              </p>
            </div>
          {/if}
        </li>
        <li class="relative">
          <div class="btn text-base space-x-2 invisible">
            <span class="icon-[fluent--dumbbell-16-regular] w-6 h-6" />
            {$i18n.t('playground.title')}
          </div>
          {#if routeItemIndex === 2}
            <div
              class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <div>
                <ArrowUpAnimate width={200} height={200} />
              </div>
            </div>
            <div
              class="absolute top-40 left-0 w-96 p-6 rounded-box bg-neutral/80 backdrop-blur"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <p class="w-full text-lg indent-8">
                {$i18n.t('tutorial.platformInfoPlatform')}
              </p>
            </div>
          {/if}
        </li>
        <li class="relative">
          <div class="btn text-base space-x-2 invisible">
            <span class="icon-[fluent--flag-16-regular] w-6 h-6" />
            {$i18n.t('games.title')}
          </div>
          {#if routeItemIndex === 3}
            <div
              class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <div>
                <ArrowUpAnimate width={200} height={200} />
              </div>
            </div>
            <div
              class="absolute top-40 left-0 w-96 p-6 rounded-box bg-neutral/80 backdrop-blur"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <p class="w-full text-lg indent-8">
                {$i18n.t('tutorial.platformInfoGames')}
              </p>
            </div>
          {/if}
        </li>
        <li class="relative">
          <div class="btn text-base space-x-2 invisible">
            <span class="icon-[fluent--megaphone-16-regular] w-6 h-6" />
            {$i18n.t('announcements.title')}
          </div>
          {#if routeItemIndex === 4}
            <div
              class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <div>
                <ArrowUpAnimate width={200} height={200} />
              </div>
            </div>
            <div
              class="absolute top-40 left-0 w-96 p-6 rounded-box bg-neutral/80 backdrop-blur"
              transition:fly={{ y: 16, duration: 500 }}
            >
              <p class="w-full text-lg indent-8">
                {$i18n.t('tutorial.platformInfoAnnouncements')}
              </p>
            </div>
          {/if}
        </li>
      </ul>
    </div>
    <h1 class="text-4xl font-bold transition-all" in:fly={{ y: 32, duration: 1000 }}>
      🫡&nbsp;{$i18n.t('tutorial.platformInfoTitle')}
    </h1>
    <div class="transition-all" in:fly={{ y: 16, duration: 1000, delay: 1500 }}>
      <RxButton size="lg" on:click={next}>
        <span class="icon-[fluent--thumb-like-16-regular] w-6 h-6"></span>
        <span>{$i18n.t('tutorial.next')}</span>
      </RxButton>
    </div>
  </div>
{/if}
