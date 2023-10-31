<script lang="ts">
  import RxForm from '$lib/components/RxForm.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { z } from 'zod'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import type { User } from '$lib/models/user'
  import { updateSelfSetting } from '$lib/api/account'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onMount } from 'svelte'
  import type { Obj } from '@felte/core'
  import { blur } from 'svelte/transition'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import { uploadMedia } from '$lib/api/media'
  import { getMediaPath } from '$lib/models/media'
  import { getUserInfo } from '$lib/api/user'
  import { user } from '$lib/stores/user'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('user.nameRequired') }),
    cover_path: z.string().trim().nullable(),
    email: z
      .string()
      .trim()
      .email($i18n.t('user.emailInvalid'))
      .min(1, { message: $i18n.t('user.emailRequired') }),
    intro: z.string().trim().nullable(),
  })
  let loading = false
  let submitting = false
  let userSetting: User = {
    id: 0,
    name: '',
    email: '',
    intro: '',
    cover_path: null,
    institute_info: null,
    institute_id: null,
    permissions: [],
    hidden: false,
    banned: false,
  }
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      submitting = true
      const payload: User = {
        ...userSetting,
        name: values.name,
        cover_path: values.cover_path,
        email: values.email,
        intro: values.intro,
      } as User
      updateSelfSetting(payload)
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
  let loadingAvatar = false
  let fileInput: HTMLInputElement

  function uploadAvatar() {
    if (fileInput.files && fileInput.files.length > 0) {
      const file = fileInput.files[0]
      uploadMedia(file, true).then((resp) => {
        const model = resp.model
        const path = getMediaPath(model)
        if (userSetting) userSetting.cover_path = path
        $data.cover_path = path
        $touched.cover_path = true
      })
    }
  }
  onMount(() => {
    loading = true
    loadingAvatar = true
    getUserInfo($user.id)
      .then((res) => {
        userSetting = res
        loadingAvatar = false
        data.update(() => {
          return userSetting as unknown as Obj
        })
      })
      .catch((error) => {
        showMessage('error', `${$i18n.t('account.fetchFailed')}: ${(error as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  })
</script>

<svelte:head><title>{$i18n.t('account.infoSetting')} - {$platform.name}</title></svelte:head>

<div class="flex-1 relative">
  <div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
    <div class="flex-1 flex flex-col max-w-5xl">
      <div class="m-2 md:m-4 flex flex-row justify-center items-center space-x-6">
        <span class="icon-[fluent--chevron-double-right-20-regular] opacity-60" />
        <h1 class="text-2xl font-bold">{$i18n.t('account.infoSetting')}</h1>
        <span class="icon-[fluent--chevron-double-left-20-regular] opacity-60" />
      </div>
      <RxForm {form}>
        <div class="flex flex-row space-x-4">
          <RxFormItem
            name="cover_path"
            label=""
            class="flex-none"
            hasError={$errors.cover_path !== null}
            errors={$errors.cover_path}
          >
            <input name="cover_path" class="hidden" value={userSetting?.cover_path} />
            <input class="hidden" type="file" bind:this={fileInput} accept="image/*" on:change={uploadAvatar} />
            <div class="avatar">
              <div
                class="w-16 h-16 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center relative"
              >
                {#if userSetting?.cover_path}
                  <RxImage src={userSetting.cover_path} loading={loadingAvatar} />
                {:else}
                  <span class="w-5 h-5 icon-[fluent--person-20-regular]" />
                {/if}
                <RxButton
                  class="absolute w-full h-full top-0 left-0 opacity-0 hover:opacity-100"
                  on:click={() => {
                    if (userSetting?.cover_path) {
                      userSetting.cover_path = null
                      $data.cover_path = null
                      $touched.cover_path = true
                    } else if (userSetting) {
                      fileInput.click()
                    }
                  }}
                >
                  {#if userSetting?.cover_path}
                    <span class="icon-[fluent--dismiss-20-regular] w-5 h-5 text-error" />
                  {:else}
                    <span class="icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5" />
                  {/if}
                </RxButton>
              </div>
            </div>
          </RxFormItem>
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
              value={userSetting.name}
            />
          </RxFormItem>
        </div>
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
            value={userSetting.email}
          />
        </RxFormItem>
        <RxFormItem
          name="intro"
          label={$i18n.t('account.intro')}
          hasError={$errors.intro !== null}
          errors={$errors.intro}
        >
          <RxCodeBox class="h-[20rem]" name="intro" hasError={$errors.intro !== null} value={userSetting.intro} />
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
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</div>
