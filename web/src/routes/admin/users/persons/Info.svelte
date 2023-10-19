<script lang="ts">
  import { getInstituteList, updateUser } from '$lib/api/user'
  import { i18n } from '$lib/i18n'
  import type { Institute } from '$lib/models/institute'
  import { Permission, permissionToString, type User } from '$lib/models/user'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { onMount } from 'svelte'
  import { z } from 'zod'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import { uploadMedia } from '$lib/api/media'
  import { getMediaPath } from '$lib/models/media'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import type { AxiosError } from 'axios'
  import { showMessage } from '$lib/stores/toast'

  export let user: User | null

  let institutes: Institute[] = []

  onMount(() => {
    if (user) {
      watchUser(user)
      getInstituteList().then((value) => {
        institutes = value
      })
    }
  })

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(2, { message: $i18n.t('account.accountTooShort') })
      .max(32, { message: $i18n.t('account.accountTooLong') }),
    email: z
      .string()
      .trim()
      .max(120, { message: $i18n.t('account.emailTooLong') })
      .email({ message: $i18n.t('account.emailInvalid') }),
    cover_path: z.string().nullable(),
    intro: z
      .string()
      .max(1500, { message: $i18n.t('account.introTooLong') })
      .nullable(),
    permissions: z.array(
      z
        .number()
        .nonnegative()
        .int()
        .min(0, { message: $i18n.t('account.permissionInvalid') })
        .max(8, { message: $i18n.t('account.permissionInvalid') })
    ),
    hidden: z.boolean(),
    banned: z.boolean(),
    institute_id: z.string().nullable(),
    institute_info: z.string().nullable(),
  })

  function watchUser(u: User | null) {
    if (!u) return
    $data = {
      ...u,
    }
    Object.keys($touched).forEach((key) => {
      $touched[key] = true
    })
    updatePermissionChecks()
  }

  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      if (values.institute_info?.trim().length === 0) values.institute_info = null
      if (values.intro?.trim().length === 0) values.intro = null
      if (values.institute_id?.trim().length === 0) values.institute_id = null
      else values.institute_id = parseInt(values.institute_id)
      if (isNaN(values.institute_id)) values.institute_id = null
      if (values.cover_path?.trim().length === 0) values.cover_path = null
      const newUser: User = {
        id: user?.id,
        ...values,
      }
      if (user)
        updateUser(user.id, newUser)
          .then(() => {
            user = newUser
            showMessage('success', $i18n.t('account.updateSuccess'), 5000)
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('account.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
    },
  })

  $: watchUser(user)

  function updatePermissionChecks() {
    if (user)
      for (let i = 0; i < permissionChecks.length; i++) {
        permissionChecks[i] = user.permissions.includes(i as Permission)
      }
  }

  let permissionChecks = [false, false, false, false, false, false, false, false, false]

  $: watchChecks(permissionChecks)

  function watchChecks(checks: boolean[]) {
    $data.permissions = checks
      .map((p, i) => {
        if (p) return i
      })
      .filter((p) => p !== undefined) as number[]
    console.log($data.permissions)
    $touched.permissions = true
  }

  let fileInput: HTMLInputElement

  function uploadAvatar() {
    if (fileInput.files && fileInput.files.length > 0) {
      const file = fileInput.files[0]
      uploadMedia(file, true).then((resp) => {
        const model = resp.model
        const path = getMediaPath(model)
        $data.cover_path = path
        $touched.cover_path = true
      })
    }
  }

  $: cover_path = $data.cover_path as string
</script>

<RxForm class="p-4 lg:p-6" {form}>
  <div class="flex flex-row space-x-4">
    <RxFormItem
      name="cover_path"
      label=""
      class="flex-none"
      hasError={$errors.cover_path !== null}
      errors={$errors.cover_path || ''}
    >
      <input name="cover_path" class="hidden" bind:value={$data.cover_path} />
      <input class="hidden" type="file" bind:this={fileInput} accept="image/*" on:change={uploadAvatar} />
      <div class="avatar mx-4">
        <div
          class="w-16 h-16 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center relative"
        >
          {#if $data.cover_path}
            <RxImage src={cover_path} loading={false} />
          {:else}
            <span class="w-5 h-5 icon-[fluent--person-20-regular]" />
          {/if}
          <RxButton
            class="absolute w-full h-full top-0 left-0 opacity-0 hover:opacity-100"
            on:click={() => {
              if ($data.cover_path) {
                $data.cover_path = null
                $touched.cover_path = true
              } else if (user) {
                fileInput.click()
              }
            }}
          >
            {#if $data.cover_path}
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
      errors={$errors.name || ''}
    >
      <RxInput
        icon="icon-[fluent--person-20-regular]"
        class="w-full"
        id="name"
        name="name"
        hasError={$errors.name !== null}
        placeholder={$i18n.t('account.name')}
        value={user?.name || ''}
      />
    </RxFormItem>
    <RxFormItem
      name="email"
      label={$i18n.t('account.email')}
      hasError={$errors.email !== null}
      errors={$errors.email || ''}
    >
      <RxInput
        icon="icon-[fluent--mail-20-regular]"
        class="w-full"
        id="email"
        name="email"
        hasError={$errors.email !== null}
        placeholder={$i18n.t('account.email')}
        value={user?.email || ''}
      />
    </RxFormItem>
  </div>
  <div class="flex flex-row space-x-4">
    <RxFormItem
      class="flex-none"
      name="hidden"
      label=""
      hasError={$errors.hidden !== null}
      errors={$errors.hidden || ''}
    >
      <RxCheckBox id="hidden" name="hidden" label={$i18n.t('account.hidden')} checked={user?.hidden} />
    </RxFormItem>
    <RxFormItem
      class="flex-none"
      name="banned"
      label=""
      hasError={$errors.banned !== null}
      errors={$errors.banned || ''}
    >
      <RxCheckBox id="banned" name="banned" label={$i18n.t('account.banned')} checked={user?.banned} />
    </RxFormItem>
  </div>
  <RxFormItem
    class="flex-none"
    name="permissions"
    label={$i18n.t('account.permissions')}
    hasError={$errors.permissions !== null}
    errors={$errors.permissions || ''}
  >
    <div class="flex flex-row space-x-4">
      <RxCheckBox label={permissionToString(0)} bind:checked={permissionChecks[0]} />
      <RxCheckBox label={permissionToString(1)} bind:checked={permissionChecks[1]} />
      <RxCheckBox label={permissionToString(2)} bind:checked={permissionChecks[2]} />
      <RxCheckBox label={permissionToString(3)} bind:checked={permissionChecks[3]} />
      <RxCheckBox label={permissionToString(4)} bind:checked={permissionChecks[4]} />
      <RxCheckBox label={permissionToString(5)} bind:checked={permissionChecks[5]} />
      <RxCheckBox label={permissionToString(6)} bind:checked={permissionChecks[6]} />
      <RxCheckBox label={permissionToString(7)} bind:checked={permissionChecks[7]} />
      <RxCheckBox label={permissionToString(8)} bind:checked={permissionChecks[8]} />
    </div>
  </RxFormItem>
  <RxFormItem
    name="intro"
    label={$i18n.t('account.intro')}
    hasError={$errors.intro !== null}
    errors={$errors.intro || ''}
  >
    <RxCodeBox
      class="w-full !h-64"
      name="intro"
      hasError={$errors.intro !== null}
      placeholder={$i18n.t('account.intro')}
      value={user?.intro || ''}
    />
  </RxFormItem>
  <div class="flex flex-row space-x-4">
    <RxFormItem
      name="institute_id"
      label={$i18n.t('account.institute_id')}
      hasError={$errors.institute_id !== null}
      errors={$errors.institute_id || ''}
      class="relative"
    >
      <RxSelect
        name="institute_id"
        availableOptions={institutes
          .map((i) => {
            return { id: i.id, label: i.name }
          }) //@ts-expect-error id is string | number | null
          .concat([{ id: null, label: 'NONE' }])}
        value={user?.institute_id || null}
      />
    </RxFormItem>
    <RxFormItem
      name="institute_info"
      label={$i18n.t('account.institute_info')}
      hasError={$errors.institute_info !== null}
      errors={$errors.institute_info || ''}
    >
      <RxInput
        icon="icon-[fluent--person-20-regular]"
        class="w-full"
        id="institute_info"
        name="institute_info"
        hasError={$errors.institute_info !== null}
        placeholder={$i18n.t('account.institute_info')}
        value={user?.institute_info || ''}
      />
    </RxFormItem>
  </div>
  <RxFormItem name="submitAction" label="">
    <RxButton class="w-full" type="submit">{$i18n.t('account.update')}</RxButton>
  </RxFormItem>
</RxForm>
