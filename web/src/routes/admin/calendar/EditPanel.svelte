<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
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

  $: classes = `absolute w-full bottom-0 flex flex-col overflow-hidden ${clazz}`

  const dispatch = createEventDispatcher()

  let schema = z.object({
    name: z.string().min(1, { message: $i18n.t('calendar.nameInvalid') }),
    intro: z.string().min(1, { message: $i18n.t('calendar.introInvalid') }),
    link: z
      .string()
      .url({ message: $i18n.t('calendar.linkInvalid') })
      .nullable(),
    start_time: z.number().int(),
    end_time: z.number().int(),
  })

  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      // TODO
    },
    onSuccess() {
      // TODO
    },
  })
</script>

<div class={classes}>
  <OverlayScrollbarsComponent
    options={{
      scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
    }}
    class="w-full h-full relative print:hidden bg-base-100/80 backdrop-blur"
    defer
  >
    <div
      class="sticky top-0 h-16 min-h-16 border-b border-b-base-content/5 backdrop-blur bg-base-100 flex flex-row px-2 items-center space-x-2"
    >
      <div class="flex-1 flex flex-row items-center px-4">
        <h1 class="text-base font-bold">{calendar.id > 0 ? $i18n.t('calendar.edit') : $i18n.t('calendar.create')}</h1>
      </div>
      <RxButton
        ghost
        level="error"
        class="join-item ml-0"
        on:click={() => {
          dispatch('close')
        }}
      >
        <span class="icon-[fluent--dismiss-16-regular] w-5 h-5"></span>
      </RxButton>
    </div>

    <RxForm class="p-4 lg:p-6" {form}>
      <RxFormItem
        name="name"
        label={$i18n.t('calendar.name')}
        hasError={$errors.name !== null}
        errors={$errors.name || ''}
      >
        <RxInput
          icon="icon-[fluent--flag-16-regular]"
          class="w-full"
          id="name"
          name="name"
          hasError={$errors.name !== null}
          placeholder={$i18n.t('calendar.name')}
        />
      </RxFormItem>
    </RxForm>
  </OverlayScrollbarsComponent>
</div>
