<script lang="ts">
  import { createCalendar, deleteCalendar, getCalendar, getCalendarList, updateCalendar } from '$lib/api/calendar'
  import type { DTColumnAction, DTColumnsDef, DTDataEntry } from '$lib/blocks/DataTable'
  import DataTable from '$lib/blocks/DataTable.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Calendar } from '$lib/models/calendar'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import EditPanel from './EditPanel.svelte'
  import { page } from '$app/stores'
  import { user } from '$lib/stores/user'
  import { platform } from '$lib/stores/platform'

  let currentPage = 0
  let total = 0
  let currentYear = new Date().getFullYear()
  let currentMonth = new Date().getMonth() + 1
  $: startTime = new Date(currentYear, currentMonth - 1, 0).getTime() / 1000
  $: endTime = new Date(currentYear, currentMonth, 0).getTime() / 1000
  let loading = false
  let loadingCalendar = false
  let submitting = false
  let activeCalendar: Calendar = {
    id: 0,
    name: '',
    intro: '',
    link: '',
    start_time: 0,
    end_time: 0,
    reporter_id: 0,
  }
  let calendars: Calendar[] = []
  let showCreatePanel = false

  let deleteModalOpened = false
  let willDeletedCalendar = {
    id: 0,
    name: '',
  }
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
        deleteModalOpened = true
        willDeletedCalendar = {
          id: data.id as number,
          name: data.name as string,
        }
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
    // console.log('before', startTime, endTime)
    getCalendarList(startTime, endTime)
      .then((res) => {
        calendars = res
        // console.log('after', startTime, endTime, res)
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
    setTimeout(() => {
      fetchCalendars()
    })
  }

  function nextMonth() {
    currentMonth += 1
    if (currentMonth === 13) {
      currentMonth = 1
      currentYear += 1
    }
    setTimeout(() => {
      fetchCalendars()
    })
  }

  onMount(() => {
    fetchCalendars()
  })

  function updateOrCreateCalendar(cal: Calendar) {
    submitting = true
    if (activeCalendar.id > 0) {
      updateCalendar(activeCalendar.id, cal)
        .then(() => {
          showMessage('success', $i18n.t('announcement.updateSuccess'), 5000)
          window.location.hash = ''
          fetchCalendars()
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('announcement.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
    } else {
      const calendar: Calendar = {
        ...cal,
        id: 0,
        reporter_id: $user.id,
      }
      createCalendar(calendar)
        .then(() => {
          showMessage('success', $i18n.t('announcement.createSuccess'), 5000)
          window.location.hash = ''
          fetchCalendars()
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('announcement.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
    }
  }

  const unsubscribe = page.subscribe((val) => {
    if (val.url.hash && val.url.hash.replace('#', '')) {
      loadingCalendar = true
      const id = parseInt(val.url.hash.replace('#', ''))
      showCreatePanel = true
      if (id && !Number.isNaN(id)) {
        getCalendar(id)
          .then((res) => {
            activeCalendar = res
            showCreatePanel = true
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('calendar.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
          .finally(() => {
            loadingCalendar = false
          })
      } else if (Number.isNaN(id)) {
        activeCalendar = {
          id: 0,
          name: '',
          intro: '',
          link: '',
          start_time: 0,
          end_time: 0,
          reporter_id: $user.id,
        }
        loadingCalendar = false
      }
    } else {
      showCreatePanel = false
      activeCalendar = {
        id: 0,
        name: '',
        intro: '',
        link: '',
        start_time: 0,
        end_time: 0,
        reporter_id: 0,
      }
    }
  })

  function handleDeleteCalendar() {
    deleteCalendar(willDeletedCalendar.id)
      .then(() => {
        showMessage('success', $i18n.t('announcement.deleteSuccess'), 5000)
        deleteModalOpened = false
        fetchCalendars()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('announcement.deleteFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  onDestroy(() => {
    unsubscribe()
  })
</script>

<svelte:head><title>{$i18n.t('admin.calendarsSettings')} - {$platform.name}</title></svelte:head>

<div class="w-full flex-1 flex flex-col relative">
  <div class="w-full flex-1 flex flex-col px-6 lg:px-12">
    <div class="h-16 flex flex-row items-center space-x-2">
      <h2 class="text-base font-bold flex-1">{$i18n.t('admin.calendarsSettings')}</h2>
      <div class="join">
        <RxButton size="sm" class="join-item" on:click={preMonth} disabled={loading}>
          <span class="icon-[fluent--chevron-double-left-20-regular] w-5 h-5"></span>
        </RxButton>
        <h2 class="text-base font-bold join-item flex flex-row items-center px-2 backdrop-blur bg-base-content/5 ml-0">
          <span>{currentYear} - {currentMonth.toString().padStart(2, '0')}</span>
        </h2>
        <RxButton size="sm" class="join-item ml-0" on:click={nextMonth} disabled={loading}>
          <span class="icon-[fluent--chevron-double-right-20-regular] w-5 h-5"></span>
        </RxButton>
      </div>
      <RxLink size="sm" level="info" href="#create">
        <span class="icon-[fluent--add-16-regular] w-6 h-6" />
        <span class="text-base">{$i18n.t('action.create')}</span>
      </RxLink>
    </div>
    <DataTable class="flex-1" {actions} data={calendars} {colDef} bind:page={currentPage} {total} {loading} />
  </div>
  <EditPanel
    class={`transition-all ${showCreatePanel ? 'h-full' : 'h-0'}`}
    bind:calendar={activeCalendar}
    loading={loadingCalendar}
    {submitting}
    on:close={() => {
      window.location.hash = ''
    }}
    on:submit={(event) => {
      updateOrCreateCalendar(event.detail)
    }}
  />
  {#if deleteModalOpened}
  <div
    class="fixed top-0 left-0 w-full h-full bg-base-100/60 z-50 flex flex-col items-center justify-center"
    transition:blur={{ amount: 20, duration: 300 }}
  >
    <div class="rounded-box p-4 flex flex-col space-y-4 bg-neutral w-64">
      <h1 class="text-base font-bold">
        {$i18n.t('form.deleteConfirm', { item: decodeURI(willDeletedCalendar.name.split('|')[0]) })}
      </h1>
      <div class="flex flex-row justify-end space-x-4">
        <RxButton size="sm" on:click={() => (deleteModalOpened = false)}>{$i18n.t('form.cancel')}</RxButton>
        <RxButton size="sm" level="error" on:click={handleDeleteCalendar}>{$i18n.t('form.confirm')}</RxButton>
      </div>
    </div>
  </div>
{/if}
</div>
