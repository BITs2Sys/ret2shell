<script lang="ts">
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxRadioGroup from '$lib/components/RxRadioGroup.svelte'
  import RxSelect from '$lib/components/RxSelect.svelte'
  import { i18n } from '$lib/i18n'
  import { State, type Team } from '$lib/models/team'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { createEventDispatcher, onMount } from 'svelte'
  import { z } from 'zod'
  import type { Institute } from '$lib/models/institute'
  import type { Obj } from '@felte/core'

  export let team: Team
  export let loading = false
  export let submitting = false
  export let institutes: Institute[] = []

  let clazz = ''
  export { clazz as class }

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('team.nameRequired') }),
    institute_id: z.string().nullable(),
    state: z.number().int().min(0).max(3),
  })

  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      if (values.institute_id?.trim().length === 0) values.institute_id = null
      else values.institute_id = parseInt(values.institute_id)
      if (isNaN(values.institute_id)) values.institute_id = null
      const newTeam: Team = {
        ...team,
        ...values,
      }
      dispatch('submit', newTeam)
    },
  })

  const stateValue = $data.state
  $: {
    if (stateValue !== $data.state) {
      $touched.state = true
    }
  }
  const dispatch = createEventDispatcher()

  onMount(() => {
    data.update(() => {
      console.log(team)
      console.log(institutes)
      let data = {
        ...team,
        institute_id: team.institute_id?.toString() || null,
      }
      return data as unknown as Obj
    })
  })
</script>

<ExtraPanel class={clazz} title={$i18n.t('team.edit')} on:close={() => dispatch('close')}>
  <RxForm class="p-4 lg:p-6" {form}>
    <RxFormItem name="name" label={$i18n.t('team.name')} hasError={$errors.name !== null} errors={$errors.name}>
      <RxInput
        name="name"
        class="w-full"
        label={$i18n.t('team.name')}
        placeholder={$i18n.t('team.namePlaceholder')}
        disabled={loading || submitting}
        value={team.name}
      />
    </RxFormItem>
    <RxFormItem
      name="institute_id"
      label={$i18n.t('account.institute_id')}
      hasError={$errors.institute_id !== null}
      errors={$errors.institute_id}
      class="relative"
    >
      <RxSelect
        name="institute_id"
        availableOptions={institutes
          .map((i) => {
            return { id: i.id, label: i.name }
          }) //@ts-expect-error id is string | number | null
          .concat([{ id: null, label: 'NONE' }])}
        value={team.institute_id}
      />
    </RxFormItem>
    <RxFormItem name="state" label={$i18n.t('team.state')} hasError={$errors.state !== null} errors={$errors.state}>
      <RxRadioGroup
        class="w-full"
        direction="row"
        items={[
          { label: $i18n.t('team.stateBanned'), value: State.Banned },
          { label: $i18n.t('team.stateNeedAudit'), value: State.NeedAudit },
          { label: $i18n.t('team.stateNormal'), value: State.Normal },
          { label: $i18n.t('team.stateHidden'), value: State.Hidden },
        ]}
        bind:value={$data.state}
      />
    </RxFormItem>
    <RxFormItem name="submitAction" label="">
      <RxButton class="w-full" type="submit" loading={submitting}>
        {submitting ? $i18n.t('team.updating') : $i18n.t('team.update')}
      </RxButton>
    </RxFormItem>
  </RxForm>
</ExtraPanel>
