<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import { goto } from '$app/navigation'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'

  let schema = z.object({
    anti_theft: z.boolean(),
    limit: z.number().min(10, { message: $i18n.t('init.mediaLimitInvalid') }),
  })

  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      initConfig.update((data) => {
        data = {
          ...data,
          config: {
            ...data.config,
            media: values,
          },
        }
        return data
      })
      return Promise.resolve()
    },
    onSuccess() {
      goto('/init/pusher')
    },
  })
</script>

<svelte:head><title>{$i18n.t('init.mediaSettings')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.mediaSettings')}</h1>
      <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem
        name="limit"
        label={$i18n.t('init.mediaLimit')}
        hasError={$errors.limit !== null}
        errors={$errors.limit}
      >
        <RxInput
          icon="icon-[fluent--code-20-regular]"
          class="w-full"
          id="limit"
          name="limit"
          hasError={$errors.limit !== null}
          placeholder={$i18n.t('init.mediaLimitPlaceholder')}
          value="25"
          type="number"
        />
      </RxFormItem>
      <RxFormItem name="anti_theft" label="" hasError={$errors.anti_theft !== null} errors={$errors.anti_theft}>
        <RxCheckBox id="anti_theft" name="anti_theft" label={$i18n.t('init.mediaAntiTheft')} />
      </RxFormItem>
      <RxFormItem name="submitAction" label="">
        <RxButton class="w-full" level="primary" type="submit">{$i18n.t('init.next')}</RxButton>
      </RxFormItem>
    </RxForm>
  </div>
</div>
