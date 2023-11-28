<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCard from '$lib/components/RxCard.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import Logo from '$lib/assets/logo.svg'
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import Captcha from '$lib/blocks/Captcha.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { login } from '$lib/api/account'
  import { goto } from '$app/navigation'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import GitHub from '$lib/assets/brands/github.svelte'
  import Google from '$lib/assets/brands/google.svelte'
  import GitLab from '$lib/assets/brands/gitlab.svelte'
  import QQ from '$lib/assets/brands/qq.svelte'
  import XDU from '$lib/assets/brands/xdu.svelte'
  import { page } from '$app/stores'

  let schema = z.object({
    account: z
      .string()
      .trim()
      .min(2, { message: $i18n.t('account.accountTooShort') })
      .max(32, { message: $i18n.t('account.accountTooLong') }),
    password: z
      .string()
      .trim()
      .min(8, { message: $i18n.t('account.passwordTooShort') })
      .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[^]{8,40}$/, { message: $i18n.t('account.passwordTooWeak') }),
    captcha_id: z.string().trim(),
    captcha_answer: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('account.captchaIsRequired') }),
  })
  let loading = false
  let captcha: Captcha | null
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      loading = true
      return login({ ...values })
    },
    onSuccess() {
      loading = false
      showMessage('success', $i18n.t('account.loginSuccess'), 5000)
      goto($page.url.searchParams.get('redirect') || '/', { replaceState: true })
    },
    onError(error) {
      loading = false
      showMessage('error', $i18n.t('account.loginFailed') + ': ' + (error as AxiosError).response?.data, 5000)
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

  interface OAuthIds {
    github: { id: string | null }
    gitlab: { id: string | null }
    google: { id: string | null }
    qq: { id: string | null }
    xdu: { id: string | null }
  }

  let oauthIds: OAuthIds = {
    github: { id: null },
    gitlab: { id: null },
    google: { id: null },
    qq: { id: null },
    xdu: { id: null },
  }

  // iterate oauthIds and find out some id that are not null
  $: haveOauthIds = Object.values(oauthIds).some((v) => v.id !== null)
</script>

<svelte:head><title>{$i18n.t('account.login')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col md:justify-center items-center p-3 lg:p-6">
  <RxCard class="flex flex-col md:flex-row w-full max-w-4xl">
    <div class="md:w-0 flex-1 flex-col">
      <h1 class="text-center font-bold text-base">{$i18n.t('account.login')}</h1>
      <RxForm {form}>
        <RxFormItem
          name="account"
          label={$i18n.t('account.account')}
          hasError={$errors.account !== null}
          errors={$errors.account}
        >
          <RxInput
            icon="icon-[fluent--person-20-regular]"
            class="w-full"
            id="account"
            name="account"
            hasError={$errors.account !== null}
            autocomplete="username"
          />
        </RxFormItem>
        <RxFormItem name="password" label="" hasError={$errors.password !== null} errors={$errors.password}>
          <span slot="label" class="text-sm font-bold opacity-60 flex flex-row justify-between w-full">
            <span>{$i18n.t('account.password')}</span>
            <a class="hover:underline" href="/account/forgot-password">{$i18n.t('account.forgotPassword')}</a>
          </span>
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
        <Captcha
          bind:this={captcha}
          hasError={$errors.captcha_answer !== null}
          errors={$errors.captcha_answer}
          bind:captchaId={$data.captcha_id}
          bind:captchaAnswer={$data.captcha_answer}
        />
        <RxFormItem name="submitAction" label="">
          <RxButton class="w-full" level="primary" type="submit" {loading}>{$i18n.t('account.login')}</RxButton>
        </RxFormItem>
      </RxForm>
    </div>
    <div class="divider md:divider-horizontal opacity-60">{$i18n.t('misc.or')}</div>
    <div class="md:w-0 flex-1 flex flex-col space-y-6">
      <div class="flex-1 md:flex flex-col justify-center items-center hidden">
        <img class="object-fit max-h-40" src={Logo} alt="Ret2Shell" />
      </div>
      <RxLink class="w-full" href="/account/register">{$i18n.t('account.registerTips')}</RxLink>
      {#if haveOauthIds}
        <div class="divider opacity-60">{$i18n.t('account.3rdAuth')}</div>
        <div class="w-full flex flex-row space-x-2 justify-between">
          {#if oauthIds.xdu.id !== null}
            <RxLink
              href={`https://ids.xidian.edu.cn/authserver/login?service=${$page.url.origin}/account/oauth%3Fprovider=xdu%26action=login`}
              square
              title="西安电子科技大学 统一身份认证"
            >
              <XDU width={32} height={32} />
            </RxLink>
          {/if}
          {#if oauthIds.github.id !== null}
            <RxLink
              href={`https://github.com/login/oauth/authorize?client_id=${oauthIds.github.id}&redirect_uri=${$page.url.origin}/account/oauth%3Fprovider=github%26action=login`}
              title="GitHub Accounts"
              square
            >
              <GitHub />
            </RxLink>
          {/if}
          {#if oauthIds.gitlab.id !== null}
            <RxLink href="" square title="GitLab Accounts"><GitLab /></RxLink>
          {/if}
          {#if oauthIds.google.id !== null}
            <RxLink href="" square title="Google Accounts"><Google /></RxLink>
          {/if}
          {#if oauthIds.qq.id !== null}
            <RxLink href="" title="QQ账号登录" square><QQ /></RxLink>
          {/if}
        </div>
      {/if}
    </div>
  </RxCard>
</div>
