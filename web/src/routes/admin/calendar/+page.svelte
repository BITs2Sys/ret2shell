<script lang="ts">
  import { getCalendarList } from '$lib/api/calendar'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Calendar } from '$lib/models/calendar'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'

  let page = 0
  let total = 0
  let currentYear = new Date().getFullYear()
  let currentMonth = new Date().getMonth() + 1
  $: startTime = new Date(currentYear, currentMonth, 0).setDate(1) / 1000
  $: endTime = new Date(currentYear, currentMonth, 0).getTime() / 1000
  let loading = false
  let calendars: Calendar[] = []

  let actions: DTColumnAction[] = [
    {
      icon: 'icon-[fluent--edit-16-regular]',
      label: '',
      level: 'info',
      type: 'link',
      href: '#{id}',
    },
    {
      icon: 'icon-[fluent--delete-16-regular]',
      label: '',
      level: 'error',
      type: 'button',
      onClick: (data: DTDataEntry) => {
        // popup delete modal
      },
    },
  ]
  let colDef: DTColumnsDef = {
    id: {
      header: 'ID',
      dimmed: true,
      type: 'number',
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    name: {
      header: $i18n.t('calendar.name'),
      type: 'plain',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    intro: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    link: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'grow',
      justify: 'text-start',
    },
    start_time: {
      header: $i18n.t('calendar.startTime'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    end_time: {
      header: $i18n.t('calendar.endTime'),
      type: 'date',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    audited: {
      header: '',
      type: 'hidden',
      dimmed: true,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    game_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
    reporter_id: {
      header: '',
      type: 'hidden',
      dimmed: false,
      sizePolicy: 'shrink',
      justify: 'text-center',
    },
  }

  function fetchCalendars() {
    loading = true
    getCalendarList(startTime, endTime)
      .then((res) => {
        calendars = res
        // console.log(calendars)
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('calendar.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  function preMonth() {
    currentMonth -= 1
    if (currentMonth === 0) {
      currentMonth = 12
      currentYear -= 1
    }
  }

  function nextMonth() {
    currentMonth += 1
    if (currentMonth === 13) {
      currentMonth = 1
      currentYear += 1
    }
  }

  onMount(() => {
    fetchCalendars()
  })

  $: {
    if (currentMonth) {
      fetchCalendars()
    }
  }
</script>

<div class="flex-1 flex flex-col items-center">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.calendarsSettings')}</h2>
      <div class="join">
        <RxButton size="sm" class="join-item" on:click={preMonth}>
          <span class="icon-[fluent--chevron-double-left-20-regular] w-5 h-5"></span>
        </RxButton>
        <h2 class="text-base font-bold join-item flex flex-row items-center px-2 backdrop-blur bg-base-content/5 ml-0">
          <span>{currentYear} - {currentMonth.toString().padStart(2, '0')}</span>
        </h2>
        <RxButton size="sm" class="join-item ml-0" on:click={nextMonth}>
          <span class="icon-[fluent--chevron-double-right-20-regular] w-5 h-5"></span>
        </RxButton>
      </div>
      <RxLink size="sm" level="info" href="#create">
        <span class="icon-[fluent--add-16-regular] w-6 h-6" />
        <span class="text-base">{$i18n.t('action.create')}</span>
      </RxLink>
    </div>
    <DataTable class="flex-1" {actions} data={calendars} {colDef} bind:page {total} {loading} />
  </div>
</div>
