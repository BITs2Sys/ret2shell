<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxForm from '$lib/components/RxForm.svelte'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import { goto } from '$app/navigation'
  import { nanoid } from 'nanoid'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
    import RxButton from '$lib/components/RxButton.svelte'

  let schema = z.object({
    signing_key: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('init.signingKeyRequired') }),
    buffer_time: z.number().min(300, { message: $i18n.t('init.bufferTimeLimit') }),
    expires_time: z.number().min(600, { message: $i18n.t('init.expiresTimeLimit') }),
  })

  let randomGeneratedKey = nanoid()

  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values, _context) {
      initConfig.update((data) => {
        data = {
          ...data,
          config: {
            auth: values,
            ...data.config,
          },
        }
        return data
      })
      return Promise.resolve()
    },
    onSuccess(_response, _context) {
      goto('/init/captcha')
    },
  })
</script>

<svelte:head><title>{$i18n.t('init.authTitle')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.authTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem
        name="signing_key"
        label={$i18n.t('init.signingKey')}
        hasError={$errors.signing_key !== null}
        errors={$errors.signing_key || ''}
      >
        <RxInput
          icon="icon-[fluent--lock-16-regular]"
          class="w-full"
          id="signing_key"
          name="signing_key"
          hasError={$errors.signing_key !== null}
          placeholder={$i18n.t('init.signingKeyPlaceholder')}
          value={randomGeneratedKey}
        />
      </RxFormItem>
      <RxFormItem
        name="expires_time"
        label={$i18n.t('init.expiresTime')}
        hasError={$errors.expires_time !== null}
        errors={$errors.expires_time || ''}
      >
        <RxInput
          icon="icon-[fluent--clock-16-regular]"
          class="w-full"
          id="expires_time"
          name="expires_time"
          hasError={$errors.expires_time !== null}
          placeholder={$i18n.t('init.expiresTimePlaceholder')}
          value=86400
          type="number"
        />
      </RxFormItem>
      <RxFormItem
        name="buffer_time"
        label={$i18n.t('init.bufferTime')}
        hasError={$errors.buffer_time !== null}
        errors={$errors.buffer_time || ''}
      >
        <RxInput
          icon="icon-[fluent--clock-16-regular]"
          class="w-full"
          id="buffer_time"
          name="buffer_time"
          hasError={$errors.buffer_time !== null}
          placeholder={$i18n.t('init.bufferTimePlaceholder')}
          value=21600
          type="number"
        />
      </RxFormItem>
      <RxFormItem name="submitAction" label="">
        <RxButton class="w-full" level="primary" type="submit">{$i18n.t('init.next')}</RxButton>
      </RxFormItem>
    </RxForm>
  </div>
</div>
