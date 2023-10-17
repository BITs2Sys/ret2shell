<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { initConfig } from '$lib/stores/init'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import { nanoid } from 'nanoid'
  import { setPlatformConfig } from '$lib/api/platform'

  let schema = z.object({
    enabled: z.boolean(),
    token: z.string().min(1, { message: $i18n.t('init.pusherTokenRequired') }),
  })

  let randomToken = nanoid()

  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      initConfig.update((data) => {
        data = {
          ...data,
          config: {
            ...data.config,
            pusher: values,
          },
        }
        return data
      })
      return Promise.resolve()
    },
    onSuccess() {
      // console.log($initConfig.config)
      setPlatformConfig(
        {
          id: 0,
          ...$initConfig.config,
        },
        $initConfig.token
      ).then(() => {
        // console.log(res)
        window.location.assign('/')
      })
    },
  })
</script>

<svelte:head><title>{$i18n.t('init.pusherTitle')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.pusherTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem name="enabled" label="" hasError={$errors.enabled !== null} errors={$errors.enabled || ''}>
        <RxCheckBox id="enabled" name="enabled" label={$i18n.t('init.pusherEnabled')} />
      </RxFormItem>
      <RxFormItem
        name="token"
        label={$i18n.t('init.pusherToken')}
        hasError={$errors.token !== null}
        errors={$errors.token || ''}
      >
        <RxInput
          icon="icon-[fluent--lock-20-regular]"
          class="w-full"
          id="token"
          name="token"
          hasError={$errors.token !== null}
          placeholder={$i18n.t('init.pusherTokenPlaceholder')}
          value={randomToken}
        />
      </RxFormItem>
      <RxFormItem name="submitAction" label="">
        <RxButton class="w-full" level="primary" type="submit">{$i18n.t('init.submit')}</RxButton>
      </RxFormItem>
    </RxForm>
  </div>
</div>
