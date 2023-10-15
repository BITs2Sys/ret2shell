<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import { getPlatformConfig, setPlatformConfig } from '$lib/api/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { Config } from '$lib/models/config'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import type { Obj } from '@felte/core'
  import { blur } from 'svelte/transition'

  let schema = z.object({
    enabled: z.boolean(),
    token: z.string().min(1, { message: $i18n.t('init.pusherTokenRequired') }),
  })

  let loading = false
  let submitting = false
  let platformConfig: Config = {}
  const { form, data, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      submitting = true
      const payload: Config = {
        ...platformConfig,
        pusher: values,
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
    },
  })
  onMount(() => {
    loading = true
    getPlatformConfig()
      .then((res) => {
        platformConfig = res
        data.update(() => {
          return res.pusher as unknown as Obj
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

<svelte:head><title>{$i18n.t('init.pusherTitle')} - {$platform.name}</title></svelte:head>
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
        <h1 class="text-2xl font-bold">{$i18n.t('init.pusherTitle')}</h1>
        <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
      </div>
      <RxForm {form}>
        <RxFormItem name="enabled" label="" hasError={$errors.enabled !== null} errors={$errors.enabled || ''}>
          <RxCheckBox
            id="enabled"
            name="enabled"
            checked={platformConfig.pusher?.enabled}
            label={$i18n.t('init.pusherEnabled')}
          />
        </RxFormItem>
        <RxFormItem
          name="token"
          label={$i18n.t('init.pusherToken')}
          hasError={$errors.token !== null}
          errors={$errors.token || ''}
        >
          <RxInput
            icon="icon-[fluent--lock-16-regular]"
            class="w-full"
            id="token"
            name="token"
            hasError={$errors.token !== null}
            value={platformConfig.pusher?.token}
            placeholder={$i18n.t('init.pusherTokenPlaceholder')}
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
