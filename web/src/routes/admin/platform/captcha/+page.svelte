<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxForm from '$lib/components/RxForm.svelte'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import RxRadioGroup from '$lib/components/RxRadioGroup.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import type { Config } from '$lib/models/config'
  import { onMount } from 'svelte'
  import { getPlatformConfig, setPlatformConfig } from '$lib/api/platform'
  import type { Obj } from '@felte/core'
  import { user } from '$lib/stores/user'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { blur } from 'svelte/transition'

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
  let loading = false
  let submitting = false
  let platformConfig: Config = {}
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      submitting = true
      const payload: Config = {
        ...platformConfig,
        captcha: values,
      } as Config
      setPlatformConfig(payload, $user.token)
        .then(() => {
          showMessage('success', $i18n.t('admin.config.saved'), 5000)
        })
        .catch((error) => {
          showMessage('error', `${$i18n.t('admin.config.saveFailed')}: ${(error as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
      return Promise.resolve()
    },
  })
  const validatorValue = $data.validator
  $: {
    if (validatorValue !== $data.validator) {
      $touched.validator = true
    }
  }
  onMount(() => {
    loading = true
    getPlatformConfig()
      .then((res) => {
        platformConfig = res
        data.update(() => {
          return res.captcha as unknown as Obj
        })
      })
      .catch((error) => {
        showMessage(
          'error',
          `${$i18n.t('admin.config.platform.fetchFailed')}: ${(error as AxiosError).response?.data}`,
          5000
        )
      })
      .finally(() => {
        loading = false
      })
  })
</script>

<svelte:head><title>{$i18n.t('init.captchaTitle')} - {$platform.name}</title></svelte:head>
<div class="flex-1 relative">
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
  <div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
    <div class="flex-1 flex flex-col max-w-5xl">
      <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
        <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
        <h1 class="text-2xl font-bold">{$i18n.t('init.captchaTitle')}</h1>
        <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
      </div>
      <RxForm {form}>
        <RxFormItem name="enabled" label="" hasError={$errors.enabled !== null} errors={$errors.enabled || ''}>
          <RxCheckBox
            id="enabled"
            name="enabled"
            checked={platformConfig.captcha?.enabled}
            label={$i18n.t('init.captchaEnabled')}
          />
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
            value={platformConfig.captcha?.difficulty}
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
          <RxButton class="w-full" level="primary" type="submit" loading={submitting}>
            {submitting ? $i18n.t('admin.config.updating') : $i18n.t('admin.config.update')}
          </RxButton>
        </RxFormItem>
      </RxForm>
      <div class="h-32"></div>
    </div>
  </div>
</div>
