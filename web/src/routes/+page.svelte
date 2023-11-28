<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxCalendar from '$lib/components/RxCalendar.svelte'
  import type { Calendar } from '$lib/models/calendar'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import Logo from '$lib/assets/logo-gray.svg'
  import { onMount } from 'svelte'
  import { getCalendar, getCalendarList } from '$lib/api/calendar'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import RxTag from '$lib/components/RxTag.svelte'
  import '$lib/styles/article.scss'
  import RxPing from '$lib/components/RxPing.svelte'
  import { getPlatformVersion } from '$lib/api/platform'
  import RxPopup from '$lib/components/RxPopup.svelte'
  import LogoAnimate from '$lib/assets/animates/logo-animate.svelte'

  let calendars: Calendar[] = []

  let year = new Date().getFullYear()
  let month = new Date().getMonth() + 1

  interface CalendarBtn {
    data: Calendar
    active: boolean
  }

  let selectedCalendar: Calendar | null = null
  let chosenDate: Date | null = null
  let gameDates: Date[] = []
  let selectedDates: Date[] = []
  let calendarBtns: CalendarBtn[] = []
  let gameDescription: Promise<string | null> = Promise.resolve(null)
  let currentDate = new Date()
  let version = 'unknown'

  $: {
    calendarBtns = calendars.map((c) => ({
      data: c,
      active:
        (chosenDate &&
          chosenDate >= new Date(new Date(c.start_time * 1000).toDateString()) &&
          chosenDate <= new Date(new Date(c.end_time * 1000).toDateString())) ||
        false,
    }))
  }
  $: {
    // gameDates = ranges in [start_time, end_time]
    gameDates = calendars
      .map((c) => {
        const start = new Date(c.start_time * 1000)
        const end = new Date(c.end_time * 1000)
        const dates = []
        for (let d = start; d <= end; d.setDate(d.getDate() + 1)) {
          dates.push(new Date(d))
        }
        return dates
      })
      .flat()
    // console.log(gameDates)
    // remove duplicate dates
    gameDates = Array.from(new Set(gameDates))
  }

  $: {
    if (selectedCalendar) {
      selectedDates = []
      const start = new Date(selectedCalendar.start_time * 1000)
      const end = new Date(selectedCalendar.end_time * 1000)
      for (let d = start; d <= end; d.setDate(d.getDate() + 1)) {
        selectedDates.push(new Date(d))
      }
      selectedDates = selectedDates
      gameDescription = render(selectedCalendar.intro)
    } else {
      selectedDates = []
    }
  }

  const render = async (content: string) => {
    let { MarkTo } = await import('$lib/markto')
    let dompurify = await import('isomorphic-dompurify')
    const markTo = new MarkTo()
    await markTo.init({ type: 'html' })
    return dompurify.sanitize((await markTo.render(content)) as string)
  }

  function fetchCalendar() {
    getCalendarList(
      Math.floor(new Date(year, month - 1).getTime() / 1000),
      Math.floor(new Date(year, month).getTime() / 1000)
    )
      .then((response) => {
        calendars = response.sort((a, b) => a.start_time - b.start_time)
      })
      .catch((err) => {
        showMessage('error', $i18n.t('calendar.failedToFetch') + ': ' + (err as AxiosError).response?.data, 5000)
      })
  }

  onMount(() => {
    fetchCalendar()
    getPlatformVersion()
      .then((res) => {
        version = res
      })
      .catch((err) => {
        showMessage(
          'error',
          $i18n.t('platform.failedToFetchPlatformVersion') + ': ' + (err as AxiosError).response?.data,
          5000
        )
      })
  })

  function fetchSelected() {
    getCalendar(selectedCalendar!.id)
      .then((response) => {
        selectedCalendar = response
      })
      .catch((err) => {
        showMessage('error', $i18n.t('calendar.failedToFetch') + ': ' + (err as AxiosError).response?.data, 5000)
      })
  }

  let calendarSection: HTMLElement

  const suprises = ['sakana', 'wdnmd']
  $: surprise = `/magic/${suprises[Math.floor(Math.random() * suprises.length)]}`
</script>

<svelte:head><title>{$platform.name}</title></svelte:head>

<div class="flex-1 relative">
  <div class="absolute h-full w-full overflow-scroll snap-mandatory snap-y">
    <section class="h-full min-h-full snap-center flex flex-col items-center justify-center relative">
      <div class="flex-1" />
      <h1 class="text-3xl font-semibold">
        &nbsp;
        <span>[&nbsp;{$platform.name}&nbsp;]</span>
        <span class="text-primary animate-ping">_</span>
      </h1>
      <a class="text-xl text-error mt-8" href={$platform.subject_url}>{$platform.subject_info}</a>
      <div class="flex-1" />
      <div class="h-24" />
      <div class="absolute bottom-4 flex flex-row flex-wrap items-center justify-center h-auto p-2">
        <p
          class="m-2 p-3 py-2 h-full backdrop-blur backdrop-brightness-100 rounded-box text-gray-500 shadow-sm border border-base-content/5 inline-flex justify-center items-center flex-wrap"
        >
          (C) 2022 - {new Date().getFullYear()}&nbsp;
          <a href={$platform.footer_url} class="link">{$platform.footer_info}</a>
          {#if !$platform.hide_maker}
            &nbsp;
            <span class="opacity-60">|</span>
            &nbsp;By&nbsp;
            <a href="https://github.com/ret2shell" class="link">
              {$i18n.t('about.maker')}
            </a>
          {/if}
          {#if $platform.record}
            &nbsp;
            <span class="opacity-60">|</span>
            &nbsp;
            <a href="https://beian.miit.gov.cn" class="link">{$platform.record}</a>
          {/if}
          <a class="btn no-animation btn-sm btn-ghost btn-square ml-2 relative" href={surprise}>
            <span class="icon-[fluent--gift-20-regular] w-5 h-5 opacity-80" />
            {#if !$platform.see_magic_category}
              <RxPing level="info" />
            {/if}
          </a>
        </p>
        <button
          class="m-2 p-3 h-full backdrop-blur backdrop-brightness-100 rounded-box text-gray-500 shadow-sm border border-base-content/5 inline-flex justify-center items-center flex-wrap space-x-2 hover:animate-pulse"
          on:click={() => {
            calendarSection.scrollIntoView({ behavior: 'smooth' })
          }}
        >
          <span class="font-semibold">{$i18n.t('calendar.scrollToView')}</span>
          <span class="icon-[fluent--chevron-double-down-20-regular] w-5 h-5 opacity-80" />
        </button>
        <RxPopup
          name="platformVersion"
          class="m-2 backdrop-blur bg-transparent backdrop-brightness-100 rounded-box text-gray-500 shadow-sm border border-base-content/5"
          popupWidth="auto"
          placement="top"
        >
          <span slot="button" class="icon-[fluent--info-20-regular] w-5 h-5"></span>
          <div class="w-max flex flex-col space-y-2">
            <div
              class="bg-neutral/30 backdrop-blur rounded-box border border-base-content/5 flex flex-row items-center space-x-4 p-4 px-8"
            >
              <LogoAnimate width={64} height={64} />
              <div class="flex flex-col space-y-1">
                <h2 class="text-2xl font-bold flex flex-row">
                  <span class="text-primary">R</span>
                  <span class="opacity-80">et</span>
                  <span class="opacity-60">&nbsp;2&nbsp;</span>
                  <span class="text-error">S</span>
                  <span class="opacity-80">hell</span>
                </h2>
                <p class="text-base font-bold opacity-60">{version}</p>
              </div>
            </div>
            <div
              class="bg-neutral/30 backdrop-blur rounded-box border border-base-content/5 flex flex-row items-center space-x-2 py-2 px-3"
            >
              <RxLink href="mailto:ret2shell@woooo.tech" ghost size="sm">
                <span class="icon-[fluent--mail-20-regular] w-5 h-5"></span>
                <span class="font-normal opacity-60">ret2shell@woooo.tech</span>
              </RxLink>
              <RxLink href="https://github.com/ret2shell" ghost size="sm" square title={$i18n.t('platform.donate')}>
                <span class="icon-[fluent--flash-sparkle-20-regular] w-5 h-5"></span>
              </RxLink>
              <RxLink href="https://github.com/ret2shell" ghost size="sm" square title={$i18n.t('platform.report')}>
                <span class="icon-[fluent--open-20-regular] w-5 h-5"></span>
              </RxLink>
            </div>
          </div>
        </RxPopup>
      </div>
    </section>
    <section class="h-full min-h-full snap-center flex flex-col p-3 md:p-6" bind:this={calendarSection}>
      <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
        <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
        <h1 class="text-2xl font-bold">{$i18n.t('calendar.title')}</h1>
        <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
      </div>
      <div class="flex flex-1 flex-col lg:flex-row lg:p-24 lg:pt-12 lg:pb-36 space-y-6 lg:space-y-0 lg:space-x-24">
        <div class="flex flex-col justify-start items-center lg:p-12 !pt-0">
          <RxCalendar
            {year}
            {month}
            selectedDays={selectedDates}
            highlightDays={gameDates}
            on:select={(event) => {
              selectedCalendar = null
              chosenDate = event.detail
            }}
            on:change-date={(event) => {
              year = event.detail.year
              month = event.detail.month
              fetchCalendar()
            }}
          />
        </div>
        <div class="flex-1 flex flex-col space-y-2">
          <div class="flex-1 relative">
            {#if selectedCalendar}
              <div
                class="absolute top-0 left-0 h-full w-full bg-base-content/5 shadow backdrop-blur rounded-box p-6 lg:p-12 space-y-6 overflow-scroll flex flex-col"
              >
                <div class="flex flex-row space-x-4 items-center">
                  <span class="icon-[fluent--flag-20-regular] text-error w-5 h-5" />
                  <span class="text-base font-bold">{selectedCalendar.name}</span>
                  <div class="flex-1" />
                  {#if currentDate > new Date(selectedCalendar.end_time * 1000)}
                    <RxTag level="error" label={$i18n.t('calendar.gameOver')} />
                  {:else if currentDate < new Date(selectedCalendar.start_time * 1000)}
                    <RxTag level="info" label={$i18n.t('calendar.gameNotStarted')} />
                  {:else}
                    <RxTag level="success" label={$i18n.t('calendar.gameOngoing')} />
                  {/if}
                  <span class="icon-[fluent--calendar-20-regular] opacity-80 w-5 h-5" />
                  <span class="text-base font-bold opacity-80">
                    {new Date(selectedCalendar.start_time * 1000).toLocaleDateString('default', {
                      year: 'numeric',
                      day: '2-digit',
                      month: '2-digit',
                    })}
                    -
                    {new Date(selectedCalendar.end_time * 1000).toLocaleDateString('default', {
                      year: 'numeric',
                      day: '2-digit',
                      month: '2-digit',
                    })}
                  </span>
                </div>
                <article class="flex-1 prose !max-w-full">
                  {#await gameDescription}
                    <div class="flex flex-row space-x-2">
                      <span class="loading loading-spinner loading-sm" />
                      <span>{$i18n.t('calendar.fetching')}</span>
                    </div>
                  {:then desc}
                    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                    {@html desc}
                  {/await}
                </article>
                <div class="flex-1" />
                <div class="flex flex-row space-x-2 justify-end">
                  <RxLink level="info" href={selectedCalendar.link}>{$i18n.t('calendar.gotoGame')}</RxLink>
                  <RxButton
                    on:click={() => {
                      selectedCalendar = null
                    }}
                  >
                    {$i18n.t('calendar.iKnow')}
                  </RxButton>
                </div>
              </div>
            {:else}
              {#if calendarBtns.length === 0}
                <div class="flex flex-col space-y-12 items-center justify-center">
                  <img src={Logo} alt="LOGO" class="w-48 h-48" />
                  <span class="text-2xl font-bold opacity-60">{$i18n.t('calendar.noGame')}</span>
                </div>
              {/if}
              <div class="absolute w-full h-full">
                <div class="w-full h-full flex flex-col space-y-2 overflow-scroll">
                  {#each calendarBtns as cal}
                    <div class="border-b border-b-base-content/5 w-full">
                      <RxButton
                        ghost
                        class="w-full"
                        justify="start"
                        on:click={() => {
                          selectedCalendar = cal.data
                          fetchSelected()
                          chosenDate = null
                        }}
                      >
                        <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
                        <span
                          class={`icon-[fluent--chevron-double-right-20-regular] w-5 h-5 animate-pulse text-primary transition-all ${
                            !cal.active && '!w-0'
                          }`}
                        />
                        <span class={`text-base ${cal.active && 'text-primary'}`}>{cal.data.name}</span>
                        <div class="flex-1" />
                        {#if currentDate > new Date(cal.data.end_time * 1000)}
                          <RxTag level="error" label={$i18n.t('calendar.gameOver')} />
                        {:else if currentDate < new Date(cal.data.start_time * 1000)}
                          <RxTag level="info" label={$i18n.t('calendar.gameNotStarted')} />
                        {:else}
                          <RxTag level="success" label={$i18n.t('calendar.gameOngoing')} />
                        {/if}
                        <span class={`text-base opacity-60 ${cal.active && '!opacity-100 text-primary'}`}>
                          {new Date(cal.data.start_time * 1000).toLocaleDateString('default', {
                            year: 'numeric',
                            day: '2-digit',
                            month: '2-digit',
                          })}
                        </span>
                      </RxButton>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </section>
  </div>
</div>
