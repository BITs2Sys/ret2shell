<script lang="ts">
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxDatePicker from '$lib/components/RxDatePicker.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Calendar } from '$lib/models/calendar'
  import { theme } from '$lib/stores/theme'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { createEventDispatcher } from 'svelte'
  import { z } from 'zod'

  export let calendar: Calendar
  export let loading = false
  export let submitting = false

  let clazz = ''
  export { clazz as class }

  $: classes = `w-full flex flex-col overflow-hidden bg-neutral/20 backdrop-blur ${clazz}`

  const dispatch = createEventDispatcher()

  let schema = z.object({
    name: z.string().min(1, { message: $i18n.t('calendar.nameRequired') }),
    intro: z.string().min(1, { message: $i18n.t('calendar.introRequired') }),
    link: z.string().url({ message: $i18n.t('calendar.linkInvalid') }),
    start_time: z
      .number()
      .int()
      .min(new Date(2014, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') })
      .max(new Date(2077, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') }),
    end_time: z
      .number()
      .int()
      .min(new Date(2014, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.endTimeLimit') })
      .max(new Date(2077, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') }),
  })

  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      const newCalendar: Calendar = {
        ...calendar,
        ...values,
      }
      dispatch('submit', newCalendar)
    },
  })

  $: watchCalendar(calendar)

  function watchCalendar(cal: Calendar) {
    if (cal && cal.id > 0) {
      $data = {
        ...cal,
      }
      // mark all keys in touched as true
      Object.keys($touched).forEach((key) => {
        $touched[key] = true
      })
    } else {
      $data = {
        name: '',
        intro: '',
        link: '',
        start_time: 0,
        end_time: 0,
      }
      Object.keys($touched).forEach((key) => {
        $touched[key] = false
      })
    }
  }
</script>

<ExtraPanel
  class={clazz}
  on:close={() => dispatch('close')}
  title={calendar.id > 0 ? $i18n.t('calendar.edit') : $i18n.t('calendar.create')}
>
  <RxForm class="p-4 lg:p-6" {form}>
    <div class="flex flex-row space-x-4">
      <RxFormItem name="name" label={$i18n.t('calendar.name')} hasError={$errors.name !== null} errors={$errors.name}>
        <RxInput
          icon="icon-[fluent--flag-20-regular]"
          class="w-full"
          id="name"
          name="name"
          hasError={$errors.name !== null}
          placeholder={$i18n.t('calendar.name')}
          disabled={loading || submitting}
          value={calendar.name}
        />
      </RxFormItem>
      <RxFormItem name="link" label={$i18n.t('calendar.link')} hasError={$errors.link !== null} errors={$errors.link}>
        <RxInput
          icon="icon-[fluent--link-20-regular]"
          class="w-full"
          id="link"
          name="link"
          hasError={$errors.link !== null}
          placeholder={$i18n.t('calendar.link')}
          disabled={loading || submitting}
          value={calendar.link}
        />
      </RxFormItem>
    </div>
    <div class="flex flex-row space-x-4">
      <RxFormItem
        class="!flex-none"
        name="start_time"
        label={$i18n.t('calendar.startTime')}
        hasError={$errors.start_time !== null || $errors.end_time !== null}
        errors={$errors.start_time || $errors.end_time}
      >
        <RxDatePicker
          selectionStartName="start_time"
          selectionEndName="end_time"
          hasError={$errors.start_time !== null || $errors.end_time !== null}
          selectionStart={calendar.start_time}
          selectionEnd={calendar.end_time}
        />
      </RxFormItem>
      <RxFormItem
        class="flex-1"
        name="intro"
        label={$i18n.t('calendar.intro')}
        hasError={$errors.intro !== null}
        errors={$errors.intro}
      >
        <RxCodeBox name="intro" hasError={$errors.intro !== null} value={calendar.intro} placeholder="Mode = Markdown"
        ></RxCodeBox>
      </RxFormItem>
    </div>
    <RxFormItem name="submitAction" label="">
      <RxButton class="w-full" type="submit">{$i18n.t('calendar.submit')}</RxButton>
    </RxFormItem>
  </RxForm>
</ExtraPanel>
