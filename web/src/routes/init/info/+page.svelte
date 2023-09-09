<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { testToken } from '$lib/api/platform'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import { goto } from '$app/navigation'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.platformNameRequired') }),
    footer_info: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.footerInfoRequired') }),
    footer_url: z
      .string()
      .trim()
      .url($i18n.t('init.invalidUrl'))
      .min(1, { message: $i18n.t('init.footerUrlRequired') }),
    subject_info: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.subjectInfoRequired') }),
    subject_url: z
      .string()
      .trim()
      .url($i18n.t('init.invalidUrl'))
      .min(1, { message: $i18n.t('init.subjectUrlRequired') }),
    record: z.string().nullable(),
    hide_maker: z.boolean(),
  })
  let tokenStored = ''
  let loading = false
  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values, _context) {
      tokenStored = values.init_token
      loading = true
      return testToken(values.init_token)
    },
    onSuccess(response, context) {
      loading = false
      if ((response as Response).status === 403) {
        context.setErrors({ init_token: $i18n.t('init.tokenInvalid') })
        return
      }
      // Do something with the returned value from `onSubmit`.
      initConfig.update((config) => {
        config.token = tokenStored
        return config
      })
      goto('/init/info')
    },
  })
</script>

<svelte:head><title>{$i18n.t('init.infoTitle')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.infoTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <RxForm {form}>
        <p class="text-center text-base">{$i18n.t('init.nameTips')}</p>
        
    </RxForm>
  </div>
</div>
