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
  import RxImageUpload from '$lib/components/RxImageUpload.svelte'
  import { showMessage } from '$lib/stores/toast'
  import { onMount } from 'svelte'
  import type { AxiosError } from 'axios'
  import type { Obj } from '@felte/core'
  import { blur } from 'svelte/transition'
  import { getGame, updateGame } from '$lib/api/game'
  import { admin } from '$lib/stores/admin'
  import type { Game } from '$lib/models/game'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import { getInstituteList } from '$lib/api/user'
  import type { Institute } from '$lib/models/institute'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import RxTimePicker from '$lib/components/RxTimePicker.svelte'

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('game.nameRequired') }),
    brief: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('game.briefRequired') }),
    cover_path: z.string().trim().nullable(),
    introduction: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('game.introductionRequired') }),
    institute_id: z.string().nullable(),
    team_size_limit: z.number().min(1, { message: $i18n.t('game.teamSizeLimitRequired') }),
    hidden: z.boolean(),
    frozen: z.boolean(),
    host_as_game: z.boolean(),
    enable_team_audit: z.boolean(),
    can_register_after_started: z.boolean(),
    start_time: z.number(),
    end_time: z.number(),
    register_time: z.number(),
    archive_time: z.number(),
    blood_award_rate: z.number().min(0).max(100),
  })
  let loading = false
  let submitting = false
  let game: Game = {
    id: 0,
    updated_at: 0,
    name: '',
    brief: '',
    introduction: '',
    start_time: 0,
    end_time: 0,
    register_time: 0,
    archive_time: 0,
    hidden: false,
    frozen: false,
    host_as_game: false,
    team_size_limit: 0,
    cover_path: null,
    enable_team_audit: false,
    can_register_after_started: false,
    institute_id: null,
    blood_award_rate: 0,
  }
  let institutes: Institute[] = []

  const { form, data, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      if (!$admin.game) return
      submitting = true
      let newGame = values
      if (values.institute_id?.trim().length === 0) newGame.institute_id = null
      else newGame.institute_id = parseInt(values.institute_id)
      if (newGame.institute_id && isNaN(newGame.institute_id)) newGame.institute_id = null
      updateGame($admin.game.id, newGame)
        .then(() => {
          showMessage('success', $i18n.t('game.saved'), 5000)
        })
        .catch((error) => {
          showMessage('error', `${$i18n.t('game.saveFailed')}: ${(error as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
      return Promise.resolve()
    },
  })
  admin.subscribe((value) => {
    if (!value.game) return
    if (value.game.id === game.id) return
    game.id = value.game.id
    loading = true
    getGame(value.game.id)
      .then((res) => {
        game = res
        data.update(() => {
          return {
            ...res,
            institute_id: res.institute_id?.toString() || null,
          } as unknown as Obj
        })
      })
      .catch((error) => {
        showMessage('error', `${$i18n.t('game.fetchFailed')}: ${(error as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  })

  onMount(() => {
    getInstituteList()
      .then((res) => {
        institutes = res
      })
      .catch((error) => {
        showMessage('error', `${$i18n.t('institute.fatchFailed')}: ${(error as AxiosError).response?.data}`, 5000)
      })
  })
</script>

<svelte:head><title>{$i18n.t('admin.gameInfo')} - {$platform.name}</title></svelte:head>
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
        <h1 class="text-2xl font-bold">{$i18n.t('admin.gameInfo')}</h1>
        <span class="icon-[fluent--chevron-double-left-16-regular] opacity-60" />
      </div>
      <RxForm {form}>
        <div class="flex flex-row space-x-4">
          <div class="flex flex-1 flex-col space-y-2">
            <RxFormItem
              name="name"
              label={$i18n.t('game.name')}
              hasError={$errors.name !== null}
              errors={$errors.name || ''}
            >
              <RxInput
                icon="icon-[fluent--flag-16-regular]"
                class="w-full"
                id="name"
                name="name"
                hasError={$errors.name !== null}
                value={game.name}
                placeholder={$i18n.t('game.namePlaceholder')}
              />
            </RxFormItem>

            <RxFormItem
              name="brief"
              label={$i18n.t('game.brief')}
              hasError={$errors.brief !== null}
              errors={$errors.brief || ''}
            >
              <RxInput
                icon="icon-[fluent--info-16-regular]"
                class="w-full"
                id="brief"
                name="brief"
                hasError={$errors.brief !== null}
                value={game.brief}
                placeholder={$i18n.t('game.briefPlaceholder')}
              />
            </RxFormItem>
            <div class="flex-1"></div>
          </div>
          <RxFormItem
            class="!flex-none"
            name="cover_path"
            label={$i18n.t('game.cover')}
            hasError={$errors.cover_path !== null}
            errors={$errors.cover_path || ''}
          >
            <RxImageUpload class="h-36" name="cover_path" value={game.cover_path || ''}></RxImageUpload>
          </RxFormItem>
        </div>
        <div class="flex flex-row space-x-4">
          <RxFormItem
            name="start_time"
            label={$i18n.t('game.startTime')}
            hasError={$errors.start_time !== null}
            errors={$errors.start_time || ''}
          >
            <RxTimePicker name="start_time" value={game.start_time}></RxTimePicker>
          </RxFormItem>
          <RxFormItem
            name="end_time"
            label={$i18n.t('game.endTime')}
            hasError={$errors.end_time !== null}
            errors={$errors.end_time || ''}
          >
            <RxTimePicker name="end_time" value={game.end_time}></RxTimePicker>
          </RxFormItem>
        </div>
        <div class="flex flex-row space-x-4">
          <RxFormItem
            name="register_time"
            label={$i18n.t('game.registerTime')}
            hasError={$errors.register_time !== null}
            errors={$errors.register_time || ''}
          >
            <RxTimePicker name="register_time" value={game.register_time}></RxTimePicker>
          </RxFormItem>
          <RxFormItem
            name="archive_time"
            label={$i18n.t('game.archiveTime')}
            hasError={$errors.archive_time !== null}
            errors={$errors.archive_time || ''}
          >
            <RxTimePicker name="archive_time" value={game.archive_time}></RxTimePicker>
          </RxFormItem>
        </div>
        <RxFormItem
          name="introduction"
          label={$i18n.t('game.introduction')}
          hasError={$errors.introduction !== null}
          errors={$errors.introduction || ''}
        >
          <RxCodeBox
            class="h-[20rem]"
            name="introduction"
            hasError={$errors.introduction !== null}
            value={game.introduction}
          />
        </RxFormItem>
        <RxFormItem
          name="institute_id"
          label={$i18n.t('game.institute_id')}
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
            value={game.institute_id}
          />
        </RxFormItem>
        <div class="flex flex-row space-x-4">
          <RxFormItem
            name="team_size_limit"
            label={$i18n.t('game.team_size_limit')}
            hasError={$errors.team_size_limit !== null}
            errors={$errors.team_size_limit || ''}
          >
            <RxInput
              icon="icon-[fluent--person-20-regular]"
              class="w-full"
              id="team_size_limit"
              name="team_size_limit"
              type="number"
              hasError={$errors.team_size_limit !== null}
              value={game.team_size_limit}
              placeholder={$i18n.t('game.team_size_limitPlaceholder')}
            />
          </RxFormItem>
          <RxFormItem
            name="blood_award_rate"
            label={$i18n.t('game.blood_award_rate')}
            hasError={$errors.blood_award_rate !== null}
            errors={$errors.blood_award_rate || ''}
          >
            <RxInput
              icon="icon-[fluent--person-20-regular]"
              class="w-full"
              id="blood_award_rate"
              name="blood_award_rate"
              type="number"
              hasError={$errors.blood_award_rate !== null}
              value={game.blood_award_rate}
              placeholder={$i18n.t('game.blood_award_ratePlaceholder')}
            />
          </RxFormItem>
        </div>
        <div class="flex flex-row space-x-4">
          <RxFormItem
            name="host_as_game"
            label=""
            hasError={$errors.host_as_game !== null}
            errors={$errors.host_as_game || ''}
          >
            <RxCheckBox name="host_as_game" label={$i18n.t('game.host_as_game')} checked={game.host_as_game} />
          </RxFormItem>
          <RxFormItem
            name="enable_team_audit"
            label=""
            hasError={$errors.enable_team_audit !== null}
            errors={$errors.enable_team_audit || ''}
          >
            <RxCheckBox
              name="enable_team_audit"
              label={$i18n.t('game.enable_team_audit')}
              checked={game.enable_team_audit}
            />
          </RxFormItem>
          <RxFormItem
            name="can_register_after_started"
            label=""
            hasError={$errors.can_register_after_started !== null}
            errors={$errors.can_register_after_started || ''}
          >
            <RxCheckBox
              name="can_register_after_started"
              label={$i18n.t('game.can_register_after_started')}
              checked={game.can_register_after_started}
            />
          </RxFormItem>
          <RxFormItem name="hidden" label="" hasError={$errors.hidden !== null} errors={$errors.hidden || ''}>
            <RxCheckBox name="hidden" label={$i18n.t('game.hidden')} checked={game.hidden} />
          </RxFormItem>
          <RxFormItem name="frozen" label="" hasError={$errors.frozen !== null} errors={$errors.frozen || ''}>
            <RxCheckBox name="frozen" label={$i18n.t('game.frozen')} checked={game.frozen} />
          </RxFormItem>
        </div>
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
