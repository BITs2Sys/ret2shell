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
  import { showMessage } from '$lib/stores/toast'
  import { getPlatformConfig, setPlatformConfig } from '$lib/api/platform'
  import { onMount } from 'svelte'
  import type { Config } from '$lib/models/config'
  import type { AxiosError } from 'axios'
  import type { Obj } from '@felte/core'
  import { user } from '$lib/stores/user'
  import { blur } from 'svelte/transition'

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
  let loading = false
  let submitting = false
  let platformConfig: Config = {}
  const { form, data, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      submitting = true
      const payload: Config = {
        ...platformConfig,
        platform: values,
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
      return Promise.resolve()
    },
  })

  onMount(() => {
    loading = true
    getPlatformConfig()
      .then((res) => {
        platformConfig = res
        data.update(() => {
          return res.platform as unknown as Obj
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

<svelte:head><title>{$i18n.t('init.infoTitle')} - {$platform.name}</title></svelte:head>
{#if loading}
  <div class="flex-1 h-full relative z-20 bg-base-100" transition:blur={{ amount: 20, duration: 300 }}>
    <div class="absolute top-0 left-0 w-full h-full flex flex-row justify-center items-center">
      <span class="loading loading-spinner loading-sm" />
    </div>
  </div>
{/if}
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 flex flex-col max-w-5xl">
    <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
      <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
      <h1 class="text-2xl font-bold">{$i18n.t('init.infoTitle')}</h1>
      <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
    </div>
    <RxForm {form}>
      <RxFormItem
        name="name"
        label={$i18n.t('init.platformName')}
        hasError={$errors.name !== null}
        errors={$errors.name || ''}
      >
        <RxInput
          icon="icon-[fluent--flag-16-regular]"
          class="w-full"
          id="name"
          name="name"
          hasError={$errors.name !== null}
          value={platformConfig.platform?.name}
          placeholder={$i18n.t('init.platformNamePlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="footer_info"
        label={$i18n.t('init.footerInfo')}
        hasError={$errors.footer_info !== null}
        errors={$errors.footer_info || ''}
      >
        <RxInput
          icon="icon-[fluent--info-16-regular]"
          class="w-full"
          id="footer_info"
          name="footer_info"
          hasError={$errors.footer_info !== null}
          value={platformConfig.platform?.footer_info}
          placeholder={$i18n.t('init.footerInfoPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="footer_url"
        label={$i18n.t('init.footerUrl')}
        hasError={$errors.footer_url !== null}
        errors={$errors.footer_url || ''}
      >
        <RxInput
          icon="icon-[fluent--link-16-regular]"
          class="w-full"
          id="footer_url"
          name="footer_url"
          hasError={$errors.footer_url !== null}
          value={platformConfig.platform?.footer_url}
          placeholder={$i18n.t('init.footerUrlPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="subject_info"
        label={$i18n.t('init.subjectInfo')}
        hasError={$errors.subject_info !== null}
        errors={$errors.subject_info || ''}
      >
        <RxInput
          icon="icon-[fluent--info-16-regular]"
          class="w-full"
          id="subject_info"
          name="subject_info"
          hasError={$errors.subject_info !== null}
          value={platformConfig.platform?.subject_info}
          placeholder={$i18n.t('init.subjectInfoPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="subject_url"
        label={$i18n.t('init.subjectUrl')}
        hasError={$errors.subject_url !== null}
        errors={$errors.subject_url || ''}
      >
        <RxInput
          icon="icon-[fluent--link-16-regular]"
          class="w-full"
          id="subject_url"
          name="subject_url"
          hasError={$errors.subject_url !== null}
          value={platformConfig.platform?.subject_url}
          placeholder={$i18n.t('init.subjectUrlPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="record"
        label={$i18n.t('init.record')}
        hasError={$errors.record !== null}
        errors={$errors.record || ''}
      >
        <RxInput
          icon="icon-[fluent--slide-record-16-regular]"
          class="w-full"
          id="record"
          name="record"
          hasError={$errors.record !== null}
          value={platformConfig.platform?.record}
          placeholder={$i18n.t('init.recordPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem name="hide_maker" label="" hasError={$errors.hide_maker !== null} errors={$errors.hide_maker || ''}>
        <RxCheckBox
          id="hide_maker"
          name="hide_maker"
          checked={platformConfig.platform?.hide_maker}
          label={$i18n.t('admin.config.platform.info.hideMaker')}
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
