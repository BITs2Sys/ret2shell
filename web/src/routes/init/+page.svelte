<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import LogoFull from '$lib/assets/logo-full.svg'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { testToken } from '$lib/api/platform'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import { goto } from '$app/navigation'
  import { showMessage } from '$lib/stores/toast'
  let schema = z.object({
    init_token: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.tokenRequired') }),
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
      // Do something with the returned value from `onSubmit`.
      initConfig.update((config) => {
        config.token = tokenStored
        config.processing = true
        return config
      })
      goto('/init/info')
    },
    onError(_errors, _context) {
      loading = false
      // Do something with the errors.
      showMessage('error', $i18n.t('init.tokenInvalid'), 5000)
    },
  })
</script>

<svelte:head><title>{$i18n.t('init.title')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl items-center justify-center">
    <img class="w-1/2" src={LogoFull} alt="Ret 2 Shell" />
    <p class="text-2xl mt-4 font-bold text-center">{$i18n.t('init.welcome1')}</p>
    <p class="text-base mt-6 opacity-80 text-center">{$i18n.t('init.welcome2')}</p>
    <p class="text-base mt-2 opacity-80 text-center">{$i18n.t('init.welcome3')}</p>
    <p class="text-base mt-2 opacity-80 text-center">{$i18n.t('init.welcome4')}</p>
    <RxForm {form} class="w-full max-w-3xl mt-12">
      <RxFormItem name="init_token" label="" hasError={$errors.init_token !== null} errors={$errors.init_token || ''}>
        <RxInput
          icon="icon-[fluent--lock-16-regular]"
          class="w-full"
          id="init_token"
          name="init_token"
          hasError={$errors.init_token !== null}
          placeholder={$i18n.t('init.tokenPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem name="submitAction" label="">
        <RxButton {loading} class="w-full" level="primary" type="submit">{$i18n.t('init.submitToken')}</RxButton>
      </RxFormItem>
    </RxForm>
    <div class="h-32" />
  </div>
</div>
