<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxCard from '$lib/components/RxCard.svelte'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import Captcha from '$lib/blocks/Captcha.svelte'
  import { showMessage } from '$lib/stores/toast'
  import { resetPassword } from '$lib/api/v1/account'
  import { onMount } from 'svelte'
  import { page } from '$app/stores'
  import { goto } from '$app/navigation'

  let schema = z
    .object({
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
  let email: string = ''
  let token: string = ''
  let captcha: Captcha | null
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      loading = true
      console.log(values)
      resetPassword({ ...values, email, token })
        .then(() => {
          loading = false
          showMessage('success', $i18n.t('email.sent'), 5000)
        })
        .catch((err) => {
          loading = false
          showMessage('error', `${$i18n.t('email.sendFailed')}: ${err.response?.data}`, 5000)
          captcha?.refreshAll()
        })
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
  onMount(() => {
    email = $page.url.searchParams.get('email') || ''
    token = $page.url.searchParams.get('token') || ''
    if (!email || !token) {
      showMessage('error', $i18n.t('email.invalidVerifyLink'), undefined)
      goto('/', { replaceState: true })
      return
    }
  })
</script>

<svelte:head><title>{$i18n.t('account.resetPassword')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col md:justify-center items-center p-3 lg:p-6">
  <RxCard class="flex flex-col md:flex-row w-full max-w-xl">
    <div class="md:w-0 flex-1 flex-col">
      <h1 class="text-center font-bold text-base">{$i18n.t('account.resetPassword')}</h1>
      <RxForm {form}>
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
          <RxButton class="w-full" level="primary" type="submit" {loading}>{$i18n.t('account.submit')}</RxButton>
        </RxFormItem>
      </RxForm>
    </div>
  </RxCard>
</div>
