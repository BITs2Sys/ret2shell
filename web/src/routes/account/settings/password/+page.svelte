<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import Captcha from '$lib/blocks/Captcha.svelte'
  import { changeUserPassword } from '$lib/api/v1/account'
  import { goto } from '$app/navigation'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { page } from '$app/stores'
  import { blur } from 'svelte/transition'

  let schema = z
    .object({
      old_password: z
        .string()
        .trim()
        .min(8, { message: $i18n.t('account.passwordTooShort') }),
      new_password: z
        .string()
        .trim()
        .min(8, { message: $i18n.t('account.passwordTooShort') })
        .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[^]{8,40}$/, { message: $i18n.t('account.passwordTooWeak') }),
      passwordConfirm: z.string().trim(),
      captcha_id: z.string().trim(),
      captcha_answer: z
        .string()
        .trim()
        .min(1, { message: $i18n.t('account.captchaIsRequired') }),
    })
    .refine((data) => data.new_password === data.passwordConfirm, {
      message: $i18n.t('account.passwordNotMatch'),
      path: ['passwordConfirm'], // path of error
    })

  let loading = false
  let captcha: Captcha | null
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      loading = true
      return changeUserPassword({ ...values })
    },
    onSuccess() {
      loading = false
      showMessage('success', $i18n.t('account.updateSuccess'), 5000)
      goto($page.url.searchParams.get('redirect') || '/')
    },
    onError(error) {
      loading = false
      showMessage('error', $i18n.t('account.updateFailed') + ': ' + (error as AxiosError).response?.data, 5000)
      captcha?.refreshAll()
    },
  })

  const captchaAnswerValue = $data.captcha_answer
  $: {
    // console.log('answer', captchaAnswerValue, $data.captcha_answer)
    if (captchaAnswerValue !== $data.captcha_answer) {
      $touched.captcha_answer = true
    }
  }

  const captchaIdValue = $data.captcha_id
  $: {
    // console.log('id', captchaIdValue, $data.captcha_id)
    if (captchaIdValue !== $data.captcha_id) {
      $touched.captcha_id = true
    }
  }
</script>

<svelte:head><title>{$i18n.t('account.login')} - {$platform.name}</title></svelte:head>

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
        <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
        <h1 class="text-2xl font-bold">{$i18n.t('account.changePassword')}</h1>
        <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
      </div>
      <RxForm {form}>
        <RxFormItem
          name="old_password"
          label={$i18n.t('account.oldPassword')}
          hasError={$errors.old_password !== null}
          errors={$errors.old_password}
        >
          <RxInput
            icon="icon-[fluent--lock-20-regular]"
            class="w-full"
            id="old_password"
            type="password"
            name="old_password"
            hasError={$errors.old_password !== null}
            autocomplete="current-password"
          />
        </RxFormItem>
        <RxFormItem
          name="new_password"
          label={$i18n.t('account.password')}
          hasError={$errors.new_password !== null}
          errors={$errors.new_password}
        >
          <RxInput
            icon="icon-[fluent--lock-20-regular]"
            class="w-full"
            id="new_password"
            type="password"
            name="new_password"
            hasError={$errors.new_password !== null}
          />
        </RxFormItem>
        <RxFormItem
          name="passwordConfirm"
          label={$i18n.t('account.passwordConfirm')}
          hasError={$errors.passwordConfirm !== null}
          errors={$errors.passwordConfirm}
        >
          <RxInput
            icon="icon-[fluent--lock-20-regular]"
            class="w-full"
            id="passwordConfirm"
            type="password"
            name="passwordConfirm"
            hasError={$errors.passwordConfirm !== null}
          />
        </RxFormItem>
        <Captcha
          bind:this={captcha}
          hasError={$errors.captcha_answer !== null}
          errors={$errors.captcha_answer}
          bind:captchaId={$data.captcha_id}
          bind:captchaAnswer={$data.captcha_answer}
        />
        <RxFormItem name="submitAction" label="">
          <RxButton class="w-full" level="primary" type="submit" {loading}>{$i18n.t('account.update')}</RxButton>
        </RxFormItem>
      </RxForm>
      <div class="h-32"></div>
    </div>
  </div>
</div>
