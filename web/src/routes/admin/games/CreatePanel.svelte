<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxImageUpload from '$lib/components/RxImageUpload.svelte'
  import RxTimePicker from '$lib/components/RxTimePicker.svelte'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxCheckBox from '$lib/components/RxCheckBox.svelte'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import { i18n } from '$lib/i18n'
  import { createEventDispatcher, onMount } from 'svelte'
  import type { Game } from '$lib/models/game'
  import type { AxiosError } from 'axios'
  import { getInstituteList } from '$lib/api/v1/user'
  import { showMessage } from '$lib/stores/toast'
  import type { Institute } from '$lib/models/institute'
  import { z } from 'zod'
  import { createForm } from 'felte'
  import { validator } from '@felte/validator-zod'
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'

  export let game: Game
  export let loading = false
  export let submitting = false
  let institutes: Institute[] = []

  let clazz = ''
  export { clazz as class }

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
    is_offline: z.boolean(),
    can_register_after_started: z.boolean(),
    start_time: z
      .number()
      .int()
      .min(new Date(2014, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') })
      .max(new Date(2077, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') }),
    end_time: z
      .number()
      .int()
      .min(new Date(2014, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.endTimeLimit') })
      .max(new Date(2077, 0, 1).getTime() / 1000, { message: $i18n.t('calendar.startTimeLimit') }),
    register_time: z.number().int(),
    archive_time: z.number().int(),
    blood_award_rate: z.number().min(0).max(100),
  })

  const { form, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      console.log(values)
      const newGame: Game = {
        ...game,
        ...values,
      }
      if (values.institute_id?.trim().length === 0) newGame.institute_id = null
      else newGame.institute_id = parseInt(values.institute_id)
      if (newGame.institute_id && isNaN(newGame.institute_id)) newGame.institute_id = null
      dispatch('submit', newGame)
    },
  })

  const dispatch = createEventDispatcher()

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

<ExtraPanel class={clazz} title={$i18n.t('game.create')} on:close={() => dispatch('close')}>
  <RxForm class="p-4 lg:p-6" {form}>
    <div class="flex flex-row space-x-4">
      <div class="flex flex-1 flex-col space-y-2">
        <RxFormItem name="name" label={$i18n.t('game.name')} hasError={$errors.name !== null} errors={$errors.name}>
          <RxInput
            icon="icon-[fluent--flag-16-regular]"
            class="w-full"
            id="name"
            name="name"
            disabled={loading || submitting}
            hasError={$errors.name !== null}
            value={game.name}
            placeholder={$i18n.t('game.namePlaceholder')}
          />
        </RxFormItem>

        <RxFormItem name="brief" label={$i18n.t('game.brief')} hasError={$errors.brief !== null} errors={$errors.brief}>
          <RxInput
            icon="icon-[fluent--info-16-regular]"
            class="w-full"
            id="brief"
            name="brief"
            disabled={loading || submitting}
            hasError={$errors.brief !== null}
            value={game.brief}
            placeholder={$i18n.t('game.briefPlaceholder')}
          />
        </RxFormItem>
        <div class="flex-1"></div>
        <div class="flex flex-row space-x-2">
          <RxFormItem
            name="host_as_game"
            label=""
            hasError={$errors.host_as_game !== null}
            errors={$errors.host_as_game}
          >
            <RxCheckBox
              name="host_as_game"
              label={$i18n.t('game.host_as_game')}
              disabled={loading || submitting}
              checked={game.host_as_game}
            />
          </RxFormItem>
          <RxFormItem
            name="enable_team_audit"
            label=""
            hasError={$errors.enable_team_audit !== null}
            errors={$errors.enable_team_audit}
          >
            <RxCheckBox
              name="enable_team_audit"
              label={$i18n.t('game.enable_team_audit')}
              disabled={loading || submitting}
              checked={game.enable_team_audit}
            />
          </RxFormItem>
          <RxFormItem
            name="can_register_after_started"
            label=""
            hasError={$errors.can_register_after_started !== null}
            errors={$errors.can_register_after_started}
          >
            <RxCheckBox
              name="can_register_after_started"
              label={$i18n.t('game.can_register_after_started')}
              disabled={loading || submitting}
              checked={game.can_register_after_started}
            />
          </RxFormItem>
          <RxFormItem name="is_offline" label="" hasError={$errors.is_offline !== null} errors={$errors.is_offline}>
            <RxCheckBox
              name="is_offline"
              label={$i18n.t('game.is_offline')}
              disabled={loading || submitting}
              checked={game.is_offline}
            />
          </RxFormItem>
          <RxFormItem name="hidden" label="" hasError={$errors.hidden !== null} errors={$errors.hidden}>
            <RxCheckBox
              name="hidden"
              label={$i18n.t('game.hidden')}
              disabled={loading || submitting}
              checked={game.hidden}
            />
          </RxFormItem>
          <RxFormItem name="frozen" label="" hasError={$errors.frozen !== null} errors={$errors.frozen}>
            <RxCheckBox
              name="frozen"
              label={$i18n.t('game.frozen')}
              disabled={loading || submitting}
              checked={game.frozen}
            />
          </RxFormItem>
        </div>
        <div class="flex-1"></div>
      </div>
      <RxFormItem
        class="!flex-none"
        name="cover_path"
        label={$i18n.t('game.cover')}
        hasError={$errors.cover_path !== null}
        errors={$errors.cover_path}
      >
        <RxImageUpload class="h-64" name="cover_path" value={game.cover_path || ''}></RxImageUpload>
      </RxFormItem>
    </div>
    <div class="flex flex-row space-x-4">
      <RxFormItem
        name="start_time"
        label={$i18n.t('game.startTime')}
        hasError={$errors.start_time !== null}
        errors={$errors.start_time}
      >
        <RxTimePicker name="start_time" value={game.start_time}></RxTimePicker>
      </RxFormItem>
      <RxFormItem
        name="end_time"
        label={$i18n.t('game.endTime')}
        hasError={$errors.end_time !== null}
        errors={$errors.end_time}
      >
        <RxTimePicker name="end_time" value={game.end_time}></RxTimePicker>
      </RxFormItem>
    </div>
    <div class="flex flex-row space-x-4">
      <RxFormItem
        name="register_time"
        label={$i18n.t('game.registerTime')}
        hasError={$errors.register_time !== null}
        errors={$errors.register_time}
      >
        <RxTimePicker name="register_time" value={game.register_time}></RxTimePicker>
      </RxFormItem>
      <RxFormItem
        name="archive_time"
        label={$i18n.t('game.archiveTime')}
        hasError={$errors.archive_time !== null}
        errors={$errors.archive_time}
      >
        <RxTimePicker name="archive_time" value={game.archive_time}></RxTimePicker>
      </RxFormItem>
    </div>
    <RxFormItem
      name="introduction"
      label={$i18n.t('game.introduction')}
      hasError={$errors.introduction !== null}
      errors={$errors.introduction}
    >
      <RxCodeBox
        class="h-[20rem]"
        name="introduction"
        disabled={loading || submitting}
        hasError={$errors.introduction !== null}
        value={game.introduction}
      />
    </RxFormItem>
    <RxFormItem
      name="institute_id"
      label={$i18n.t('game.institute_id')}
      hasError={$errors.institute_id !== null}
      errors={$errors.institute_id}
      class="relative"
    >
      <RxSelect
        name="institute_id"
        disabled={loading || submitting}
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
        errors={$errors.team_size_limit}
      >
        <RxInput
          icon="icon-[fluent--person-20-regular]"
          class="w-full"
          id="team_size_limit"
          name="team_size_limit"
          type="number"
          disabled={loading || submitting}
          hasError={$errors.team_size_limit !== null}
          value={game.team_size_limit}
          placeholder={$i18n.t('game.team_size_limitPlaceholder')}
        />
      </RxFormItem>
      <RxFormItem
        name="blood_award_rate"
        label={$i18n.t('game.blood_award_rate')}
        hasError={$errors.blood_award_rate !== null}
        errors={$errors.blood_award_rate}
      >
        <RxInput
          icon="icon-[fluent--person-20-regular]"
          class="w-full"
          id="blood_award_rate"
          name="blood_award_rate"
          type="number"
          disabled={loading || submitting}
          hasError={$errors.blood_award_rate !== null}
          value={game.blood_award_rate}
          placeholder={$i18n.t('game.blood_award_ratePlaceholder')}
        />
      </RxFormItem>
    </div>
    <RxFormItem name="submitAction" label="">
      <RxButton class="w-full" type="submit" loading={submitting}>
        {submitting ? $i18n.t('game.creating') : $i18n.t('game.create')}
      </RxButton>
    </RxFormItem>
  </RxForm>
</ExtraPanel>
