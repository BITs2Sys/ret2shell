<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCard from '$lib/components/RxCard.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import Captcha from '$lib/blocks/Captcha.svelte'
  import { register } from '$lib/api/v1/account'
  import { goto } from '$app/navigation'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  let schema = z
    .object({
      name: z
        .string()
        .trim()
        .min(2, { message: $i18n.t('account.accountTooShort') })
        .max(32, { message: $i18n.t('account.accountTooLong') }),
      email: z
        .string()
        .trim()
        .max(120, { message: $i18n.t('account.emailTooLong') })
        .email({ message: $i18n.t('account.emailInvalid') }),
      password: z
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
    .refine((data) => data.password === data.passwordConfirm, {
      message: $i18n.t('account.passwordNotMatch'),
      path: ['passwordConfirm'], // path of error
    })
  let loading = false
  let captcha: Captcha | null
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      loading = true
      return register({ ...values })
    },
    onSuccess() {
      loading = false
      goto('/magic/tutorial', { replaceState: true })
    },
    onError(error) {
      loading = false
      showMessage('error', $i18n.t('account.registerFailed') + ': ' + (error as AxiosError).response?.data, 5000)
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

<svelte:head><title>{$i18n.t('account.register')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col md:justify-center items-center p-3 lg:p-6">
  <RxCard class="flex flex-col md:flex-row w-full max-w-3xl">
    <div class="md:w-0 flex-1 flex-col">
      <h1 class="text-center font-bold text-base">{$i18n.t('account.register')}</h1>
      <RxForm {form}>
        <div class="flex md:flex-row space-x-4">
          <RxFormItem
            name="name"
            label={$i18n.t('account.name')}
            hasError={$errors.name !== null}
            errors={$errors.name}
          >
            <RxInput
              icon="icon-[fluent--person-20-regular]"
              class="w-full"
              id="name"
              name="name"
              hasError={$errors.name !== null}
              autocomplete="username"
            />
          </RxFormItem>
          <RxFormItem
            name="email"
            label={$i18n.t('account.email')}
            hasError={$errors.email !== null}
            errors={$errors.email}
          >
            <RxInput
              icon="icon-[fluent--mail-20-regular]"
              class="w-full"
              id="email"
              name="email"
              hasError={$errors.email !== null}
              autocomplete="email"
            />
          </RxFormItem>
        </div>
        <RxFormItem
          name="password"
          label={$i18n.t('account.password')}
          hasError={$errors.password !== null}
          errors={$errors.password}
        >
          <RxInput
            icon="icon-[fluent--lock-20-regular]"
            class="w-full"
            id="password"
            type="password"
            name="password"
            hasError={$errors.password !== null}
            autocomplete="current-password"
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
          <RxButton class="w-full" level="primary" type="submit" {loading}>{$i18n.t('account.register')}</RxButton>
        </RxFormItem>
      </RxForm>
    </div>
  </RxCard>
</div>
