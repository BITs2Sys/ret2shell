<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import { i18n } from '$lib/i18n'
  import { createEventDispatcher } from 'svelte'
  import { z } from 'zod'
  import { createForm } from 'felte'
  import { validator } from '@felte/validator-zod'
  import ExtraPanel from '$lib/blocks/ExtraPanel.svelte'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import RxRadioGroup from '$lib/components/RxRadioGroup.svelte'

  export let challenge: Challenge
  export let loading = false
  export let submitting = false
  export let tags: Tag[]

  let clazz = ''
  export { clazz as class }

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('challenge.nameRequired') }),
    content: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('challenge.contentRequired') }),
    tag_id: z.number(),
    initial_score: z.number().min(1, { message: $i18n.t('challenge.initialScoreRequired') }),
    minimum_score: z.number().min(1, { message: $i18n.t('challenge.minimumScoreRequired') }),
    decay: z.number().min(1),
  })

  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      const newChallenge: Challenge = {
        ...challenge,
        ...values,
      }
      newChallenge.tag_id = parseInt(values.tag_id)
      dispatch('submit', newChallenge)
    },
  })

  const dispatch = createEventDispatcher()

  const tagIdValue = $data.tag_id
  $: {
    if (tagIdValue !== $data.tag_id) {
      $touched.tag_id = true
    }
  }
</script>

<ExtraPanel class={clazz} title={$i18n.t('game.create')} on:close={() => dispatch('close')}>
  <RxForm class="p-4 lg:p-6" {form}>
    <RxFormItem name="name" label={$i18n.t('challenge.name')} hasError={$errors.name !== null} errors={$errors.name}>
      <RxInput
        icon="icon-[fluent--flag-16-regular]"
        class="w-full"
        id="name"
        name="name"
        disabled={loading || submitting}
        hasError={$errors.name !== null}
        value={challenge.name}
        placeholder={$i18n.t('challenge.name')}
      />
    </RxFormItem>
    <RxFormItem
      name="content"
      label={$i18n.t('challenge.content')}
      hasError={$errors.content !== null}
      errors={$errors.content}
    >
      <RxCodeBox
        class="h-[16rem]"
        name="content"
        disabled={loading || submitting}
        hasError={$errors.content !== null}
        value={challenge.content}
      />
    </RxFormItem>
    <RxFormItem
      name="tag_id"
      label={$i18n.t('challenge.tag_id')}
      hasError={$errors.tag_id !== null}
      errors={$errors.tag_id}
      class="relative"
    >
      <!-- <RxSelect
        name="tag_id"
        disabled={loading || submitting}
        availableOptions={tags
          .map((i) => {
            return { id: i.id, label: i.name }
          }) //@ts-expect-error id is string | number | null
          .concat([{ id: null, label: 'NONE' }])}
        value={challenge.tag_id}
      /> -->
      <RxRadioGroup
        class="w-full"
        direction="row"
        items={tags.map((i) => {
          return { value: i.id, label: i.name }
        })}
        bind:value={$data.tag_id}
      />
    </RxFormItem>
    <div class="flex flex-row space-x-4">
      <RxFormItem
        name="initial_score"
        label={$i18n.t('challenge.initial_score')}
        hasError={$errors.initial_score !== null}
        errors={$errors.initial_score}
      >
        <RxInput
          icon="icon-[fluent--person-20-regular]"
          class="w-full"
          id="initial_score"
          name="initial_score"
          type="number"
          disabled={loading || submitting}
          hasError={$errors.initial_score !== null}
          value={challenge.initial_score}
        />
      </RxFormItem>
      <RxFormItem
        name="minimum_score"
        label={$i18n.t('challenge.minimum_score')}
        hasError={$errors.minimum_score !== null}
        errors={$errors.minimum_score}
      >
        <RxInput
          icon="icon-[fluent--person-20-regular]"
          class="w-full"
          id="minimum_score"
          name="minimum_score"
          type="number"
          disabled={loading || submitting}
          hasError={$errors.minimum_score !== null}
          value={challenge.minimum_score}
        />
      </RxFormItem>
      <RxFormItem
        name="decay"
        label={$i18n.t('challenge.decay')}
        hasError={$errors.decay !== null}
        errors={$errors.decay}
      >
        <RxInput
          icon="icon-[fluent--person-20-regular]"
          class="w-full"
          id="decay"
          name="decay"
          type="number"
          disabled={loading || submitting}
          hasError={$errors.decay !== null}
          value={challenge.decay}
        />
      </RxFormItem>
    </div>
    <RxFormItem name="submitAction" label="">
      <RxButton class="w-full" type="submit" loading={submitting}>
        {submitting ? $i18n.t('challenge.creating') : $i18n.t('challenge.create')}
      </RxButton>
    </RxFormItem>
  </RxForm>
</ExtraPanel>
