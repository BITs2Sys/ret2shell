<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { createEventDispatcher } from 'svelte'
  import RxButton from './RxButton.svelte'

  let clazz = ''
  export { clazz as class }
  $: classes = ['flex', 'flex-col', 'rounded-lg', 'bg-base-content/5', 'backdrop-blur', 'shadow', clazz]
    .filter(Boolean)
    .join(' ')

  export let year = new Date().getFullYear()
  export let month = new Date().getMonth() + 1
  export let selectedDays: Date[] = []

  export let highlightDays: Date[] = [new Date()]

  const dispatch = createEventDispatcher()

  function handleSelectDate(date: Date) {
    dispatch('select', date)
  }

  function handleSetMonth(m: number) {
    month = m
    if (month < 1) {
      year = year + Math.floor(month / 12) - 1
      month = 12 + month
    } else if (month > 12) {
      year = year + Math.floor(month / 12)
      month = month % 12
    }
    dispatch('change-date', { year, month })
  }

  function handleSetYear(y: number) {
    year = y
    dispatch('change-date', { year, month })
  }

  interface DayImpl {
    day: string
    date: Date
    isCurrentMonth: boolean
    highlight: boolean
    selected: boolean
  }

  class CalendarImpl {
    year: number
    month: number
    days: DayImpl[]
    constructor(year: number, month: number, highlightDays: Date[] = [], selectedDays: Date[] = []) {
      this.year = year
      this.month = month
      this.days = []
      this.init(highlightDays, selectedDays)
    }

    init(highlightDays: Date[], selectedDays: Date[]) {
      const firstDay = new Date(this.year, this.month - 1, 1)
      const lastDay = new Date(this.year, this.month, 0)
      const firstDayOfWeek = firstDay.getDay()
      const lastDayOfWeek = lastDay.getDay()
      const prevMonthDay = new Date(this.year, this.month - 1, 0).getDate()
      let prevMonthStartDay = prevMonthDay - firstDayOfWeek + 1
      let nextMonthDay = 1
      const days = []
      for (let i = 0; i < firstDayOfWeek; i++) {
        const day = new Date(this.year, this.month - 2, prevMonthStartDay).toDateString()
        days.push({
          day: prevMonthStartDay.toString().padStart(2, '0'),
          date: new Date(this.year, this.month - 2, prevMonthStartDay),
          isCurrentMonth: false,
          highlight: highlightDays.some((d) => d.toDateString() == day),
          selected: selectedDays.some((d) => d.toDateString() === day),
        })
        prevMonthStartDay++
      }
      for (let i = 1; i <= lastDay.getDate(); i++) {
        const day = new Date(this.year, this.month - 1, i).toDateString()
        days.push({
          day: i.toString().padStart(2, '0'),
          date: new Date(this.year, this.month - 1, i),
          isCurrentMonth: true,
          highlight: highlightDays.some((d) => d.toDateString() == day),
          selected: selectedDays.some((d) => d.toDateString() === day),
        })
      }
      for (let i = lastDayOfWeek; i < 6; i++) {
        const day = new Date(this.year, this.month, nextMonthDay).toDateString()
        days.push({
          day: nextMonthDay.toString().padStart(2, '0'),
          date: new Date(this.year, this.month, nextMonthDay),
          isCurrentMonth: false,
          highlight: highlightDays.some((d) => d.toDateString() == day),
          selected: selectedDays.some((d) => d.toDateString() === day),
        })
        nextMonthDay++
      }
      this.days = days
    }
  }

  $: days = new CalendarImpl(year, month, highlightDays, selectedDays).days
</script>

<div class={classes}>
  <div class="h-16 flex flex-row items-center justify-between p-2 border-b border-b-base-content/5">
    <div class="flex flex-row space-x-2">
      <RxButton ghost on:click={() => handleSetYear(year - 1)}>
        <span class="icon-[fluent--chevron-double-left-16-regular] w-6 h-6" />
      </RxButton>
      <RxButton ghost on:click={() => handleSetMonth(month - 1)}>
        <span class="icon-[fluent--chevron-left-16-regular] w-6 h-6" />
      </RxButton>
    </div>
    <RxButton
      ghost
      on:click={() => {
        handleSetMonth(new Date().getMonth() + 1)
        handleSetYear(new Date().getFullYear())
      }}
    >
      {year} - {month.toString().padStart(2, '0')}
    </RxButton>
    <div class="flex flex-row space-x-2">
      <RxButton ghost on:click={() => handleSetMonth(month + 1)}>
        <span class="icon-[fluent--chevron-right-16-regular] w-6 h-6" />
      </RxButton>
      <RxButton ghost on:click={() => handleSetYear(year + 1)}>
        <span class="icon-[fluent--chevron-double-right-16-regular] w-6 h-6" />
      </RxButton>
    </div>
  </div>
  <div class="h-16 grid grid-cols-7 p-2 border-b border-b-base-content/5">
    {#each [0, 1, 2, 3, 4, 5, 6] as day}
      <div class="flex flex-col justify-center items-center">
        <span class="text-base font-bold">{$i18n.t(`calendar.${day}`)}</span>
      </div>
    {/each}
  </div>
  <div class="flex-1 grid grid-cols-7 p-2">
    {#each days as day}
      <div class="p-2">
        <RxButton ghost={!day.selected} class="w-full h-full" on:click={() => handleSelectDate(day.date)}>
          <div class="flex flex-col justify-center items-center relative">
            {#if day.isCurrentMonth}
              <span class={`text-base ${day.selected && 'text-primary'}`}>
                {day.day}
              </span>
            {:else}
              <span class="text-base font-normal opacity-60">{day.day}</span>
            {/if}
            <div class="h-1 flex flex-row justify-center items-center space-x-1 w-4">
              {#if day.date.toLocaleDateString() === new Date().toLocaleDateString()}
                <div class="flex-1 bg-primary h-1 rounded-full" />
              {/if}
              {#if day.highlight}
                <div class="w-1 bg-primary h-1 rounded-full" />
              {/if}
            </div>
          </div>
        </RxButton>
      </div>
    {/each}
  </div>
</div>
