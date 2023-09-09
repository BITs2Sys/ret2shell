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
  import RxTextarea from '$lib/components/RxTextarea.svelte'

  let schema = z.object({
    enabled: z.boolean(),
    host: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailHostRequired') }),
    port: z
      .number()
      .min(1, { message: $i18n.t('init.emailPortRequired') })
      .max(65535, { message: $i18n.t('init.emailPortInvalid') }),
    username: z
      .string()
      .trim()
      .email($i18n.t('init.emailInvalid'))
      .min(1, { message: $i18n.t('init.emailUsernameRequired') }),
    sender: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailSenderRequired') }),
    password: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailPasswordRequired') }),
    tls: z.enum(['none', 'starttls', 'tls'], {
      errorMap: (issue, ctx) => ({ message: $i18n.t('init.tlsOptionInvalid') }),
    }),
    reset_password_email_body: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailResetPasswordBodyRequired') }),
    reset_password_email_subject: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailResetPasswordSubjectRequired') }),
    verify_email_body: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailVerifyBodyRequired') }),
    verify_email_subject: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.emailVerifySubjectRequired') }),
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
  const tlsValue = $data.tls
  $: {
    if (tlsValue !== $data.tls) {
      $touched.tls = true
    }
  }
</script>

<svelte:head><title>{$i18n.t('init.emailTitle')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.emailTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem name="enabled" label="" hasError={$errors.enabled !== null} errors={$errors.enabled || ''}>
        <RxCheckBox id="enabled" name="enabled" label={$i18n.t('init.emailEnabled')} />
      </RxFormItem>
      <div class="flex flex-row space-x-4">
        <RxFormItem
          name="host"
          class="flex-1"
          label={$i18n.t('init.emailHost')}
          hasError={$errors.host !== null}
          errors={$errors.host || ''}
        >
          <RxInput
            icon="icon-[fluent--mail-16-regular]"
            class="w-full"
            id="host"
            name="host"
            hasError={$errors.host !== null}
            placeholder={$i18n.t('init.emailHostPlaceholder')}
          />
        </RxFormItem>
        <RxFormItem
          class="!flex-none"
          name="port"
          label={$i18n.t('init.emailPort')}
          hasError={$errors.port !== null}
          errors={$errors.port || ''}
        >
          <RxInput
            icon="icon-[fluent--code-16-regular]"
            class="w-full"
            id="port"
            name="port"
            hasError={$errors.port !== null}
            placeholder={$i18n.t('init.emailPortPlaceholder')}
            value="25"
            type="number"
          />
        </RxFormItem>
      </div>
      <RxFormItem
        name="tls"
        label={$i18n.t('init.emailTls')}
        hasError={$errors.tls !== null}
        errors={$errors.tls || ''}
      >
        <RxRadioGroup
          class="w-full"
          direction="row"
          items={[
            { label: $i18n.t('init.emailNoneTls'), value: 'none' },
            { label: $i18n.t('init.emailCommonTls'), value: 'tls' },
            { label: $i18n.t('init.emailStartTls'), value: 'starttls' },
          ]}
          bind:value={$data.tls}
        />
      </RxFormItem>
      <RxFormItem
        name="sender"
        class="flex-1"
        label={$i18n.t('init.emailSender')}
        hasError={$errors.sender !== null}
        errors={$errors.sender || ''}
      >
        <RxInput
          icon="icon-[fluent--mail-16-regular]"
          class="w-full"
          id="sender"
          name="sender"
          hasError={$errors.sender !== null}
          placeholder={$i18n.t('init.emailSenderPlaceholder')}
        />
      </RxFormItem>
      <div class="flex flex-row space-x-4">
        <RxFormItem
          name="username"
          class="flex-1"
          label={$i18n.t('init.emailUsername')}
          hasError={$errors.username !== null}
          errors={$errors.username || ''}
        >
          <RxInput
            icon="icon-[fluent--mail-16-regular]"
            class="w-full"
            id="username"
            name="username"
            hasError={$errors.username !== null}
            placeholder={$i18n.t('init.emailUsernamePlaceholder')}
          />
        </RxFormItem>
        <RxFormItem
          name="password"
          class="flex-1"
          label={$i18n.t('init.emailPassword')}
          hasError={$errors.password !== null}
          errors={$errors.password || ''}
        >
          <RxInput
            icon="icon-[fluent--lock-16-regular]"
            class="w-full"
            id="password"
            name="password"
            hasError={$errors.password !== null}
            placeholder={$i18n.t('init.emailPasswordPlaceholder')}
            type="password"
          />
        </RxFormItem>
      </div>
      <div class="divider pt-12 pb-2">{$i18n.t('init.emailResetPassword')}</div>
      <RxFormItem
        name="reset_password_email_subject"
        class="flex-1"
        label={$i18n.t('init.emailResetPasswordSubject')}
        hasError={$errors.reset_password_email_subject !== null}
        errors={$errors.reset_password_email_subject || ''}
      >
        <RxInput
          class="w-full"
          id="reset_password_email_subject"
          name="reset_password_email_subject"
          hasError={$errors.reset_password_email_subject !== null}
          placeholder={$i18n.t('init.emailResetPasswordSubjectPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="reset_password_email_body"
        class="flex-1"
        label={$i18n.t('init.emailResetPasswordBody')}
        hasError={$errors.reset_password_email_body !== null}
        errors={$errors.reset_password_email_body || ''}
      >
        <RxTextarea
          id="reset_password_email_body"
          name="reset_password_email_body"
          hasError={$errors.reset_password_email_body !== null}
          placeholder={$i18n.t('init.emailResetPasswordBodyPlaceholder')}
          droppable={false}
        />
      </RxFormItem>
      <p class="p-1 text-sm opacity-60">{$i18n.t('init.emailTemplateRenderTips')}</p>
      <div class="divider pt-12 pb-2">{$i18n.t('init.emailVerifyEmail')}</div>
      <RxFormItem
        name="verify_email_subject"
        class="flex-1"
        label={$i18n.t('init.emailVerifyEmailSubject')}
        hasError={$errors.verify_email_subject !== null}
        errors={$errors.verify_email_subject || ''}
      >
        <RxInput
          class="w-full"
          id="verify_email_subject"
          name="verify_email_subject"
          hasError={$errors.verify_email_subject !== null}
          placeholder={$i18n.t('init.emailVerifyEmailSubjectPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="verify_email_body"
        class="flex-1"
        label={$i18n.t('init.emailVerifyEmailBody')}
        hasError={$errors.verify_email_body !== null}
        errors={$errors.verify_email_body || ''}
      >
        <RxTextarea
          id="verify_email_body"
          name="verify_email_body"
          hasError={$errors.verify_email_body !== null}
          placeholder={$i18n.t('init.emailVerifyEmailBodyPlaceholder')}
          droppable={false}
        />
      </RxFormItem>
      <p class="p-1 text-sm opacity-60">{$i18n.t('init.emailTemplateRenderTips')}</p>
      <RxFormItem name="submitAction" label="">
        <RxButton class="w-full" level="primary" type="submit">{$i18n.t('init.next')}</RxButton>
      </RxFormItem>
    </RxForm>
  </div>
</div>
