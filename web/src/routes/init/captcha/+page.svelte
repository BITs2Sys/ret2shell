<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'

  import RxForm from '$lib/components/RxForm.svelte'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import { goto } from '$app/navigation'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import RxRadioGroup from '$lib/components/RxRadioGroup.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import type { Validator } from '$lib/models/config'

  let schema = z.object({
    enabled: z.boolean(),
    difficulty: z
      .number()
      .min(1, { message: $i18n.t('init.captchaDifficultyMinLimit') })
      .max(9, { message: $i18n.t('init.captchaDifficultyMaxLimit') }),
    validator: z
      .number()
      .min(1, { message: $i18n.t('init.captchaValidatorLimit') })
      .max(2, { message: $i18n.t('init.captchaValidatorLimit') }),
  })

  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values, _context) {
      initConfig.update((data) => {
        data = {
          ...data,
          config: {
            captcha: {
              validator: parseInt(values.validator) as Validator,
              ...values,
            },
            ...data.config,
          },
        }
        return data
      })
      return Promise.resolve()
    },
    onSuccess(_response, _context) {
      goto('/init/email')
    },
  })
  const validatorValue = $data.validator
  $: {
    if (validatorValue !== $data.validator) {
      $touched.validator = true
    }
  }
</script>

<svelte:head><title>{$i18n.t('init.captchaTitle')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.captchaTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem name="enabled" label="" hasError={$errors.enabled !== null} errors={$errors.enabled || ''}>
        <RxCheckBox id="enabled" name="enabled" label={$i18n.t('init.captchaEnabled')} />
      </RxFormItem>
      <RxFormItem
        name="difficulty"
        label={$i18n.t('init.captchaDifficulty')}
        hasError={$errors.difficulty !== null}
        errors={$errors.difficulty || ''}
      >
        <RxInput
          icon="icon-[fluent--hand-left-16-regular]"
          class="w-full"
          id="difficulty"
          name="difficulty"
          hasError={$errors.difficulty !== null}
          placeholder={$i18n.t('init.captchaDifficultyPlaceholder')}
          value="4"
          type="number"
        />
      </RxFormItem>
      <RxFormItem
        name="validator"
        label={$i18n.t('init.captchaValidator')}
        hasError={$errors.validator !== null}
        errors={$errors.validator || ''}
      >
        <RxRadioGroup
          class="w-full"
          direction="row"
          items={[
            { label: $i18n.t('init.captchaValidatorImage'), value: 1 },
            { label: $i18n.t('init.captchaValidatorPow'), value: 2 },
          ]}
          bind:value={$data.validator}
        />
      </RxFormItem>
      <RxFormItem name="submitAction" label="">
        <RxButton class="w-full" level="primary" type="submit">{$i18n.t('init.next')}</RxButton>
      </RxFormItem>
    </RxForm>
  </div>
</div>
