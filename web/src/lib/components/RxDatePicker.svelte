<script lang="ts">
  import { createField } from 'felte'
  import RxCalendar from './RxCalendar.svelte'

  let year = new Date().getFullYear()
  let month = new Date().getMonth() + 1

  export let selectionStart = 0
  export let selectionEnd = 0
  export let selectionStartName = 'start_time'
  export let selectionEndName = 'end_time'
  export let hasError = false

  const startField = createField(selectionStartName)
  const endField = createField(selectionEndName)

  $: selectedDays = Array.from(
    { length: Math.floor((selectionEnd - selectionStart) / 86400) + 1 },
    (_, i) => new Date((selectionStart + i * 86400) * 1000)
  )

  let selectingEnd = false
  function selectDate(date: Date) {
    if (selectingEnd) {
      selectionEnd = Math.floor(date.getTime() / 1000)
      if (selectionEnd < selectionStart) {
        ;[selectionStart, selectionEnd] = [selectionEnd, selectionStart]
      }
      selectingEnd = false
      startField.onInput(selectionStart)
      endField.onInput(selectionEnd)
      startField.onBlur()
      endField.onBlur()
    } else {
      selectionStart = Math.floor(date.getTime() / 1000)
      selectionEnd = 0
      selectingEnd = true
      startField.onInput(selectionStart)
      endField.onInput(selectionEnd)
    }
  }
</script>

<RxCalendar
  class={`${hasError ? 'border border-error' : ''}`}
  {year}
  {month}
  on:change-date={(event) => {
    year = event.detail.year
    month = event.detail.month
  }}
  on:select={(event) => {
    selectDate(event.detail)
  }}
  {selectedDays}
  highlightDays={[new Date(selectionStart * 1000), new Date(selectionEnd * 1000)]}
/>
<input class="hidden" type="number" use:startField.field value={selectionStart} />
<input class="hidden" type="number" use:endField.field value={selectionEnd} />
