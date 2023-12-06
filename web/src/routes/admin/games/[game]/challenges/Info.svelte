<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import { i18n } from '$lib/i18n'
  import { onMount } from 'svelte'
  import { z } from 'zod'
  import { createForm } from 'felte'
  import { validator } from '@felte/validator-zod'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import RxRadioGroup from '$lib/components/RxRadioGroup.svelte'
  import { updateChallenge } from '$lib/api/v1/challenge'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  export let challenge: Challenge
  let submitting = false
  export let tags: Tag[] = []

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

      updateChallenge(challenge.id, newChallenge)
        .then(() => {
          showMessage('success', $i18n.t('challenge.updateSuccess'), 5000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('challenge.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
        .finally(() => {
          submitting = false
        })
    },
  })
  const tagIdValue = $data.tag_id
  $: {
    if (tagIdValue !== $data.tag_id) {
      $touched.tag_id = true
    }
  }

  onMount(() => {
    $data = {
      ...challenge,
    }
  })
</script>

<RxForm class="p-4 lg:p-6" {form}>
  <RxFormItem name="name" label={$i18n.t('challenge.name')} hasError={$errors.name !== null} errors={$errors.name}>
    <RxInput
      icon="icon-[fluent--flag-16-regular]"
      class="w-full"
      id="name"
      name="name"
      disabled={submitting}
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
      disabled={submitting}
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
        disabled={submitting}
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
        disabled={submitting}
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
        disabled={submitting}
        hasError={$errors.decay !== null}
        value={challenge.decay}
      />
    </RxFormItem>
  </div>
  <RxFormItem name="submitAction" label="">
    <RxButton class="w-full" type="submit" loading={submitting}>
      {submitting ? $i18n.t('challenge.updating') : $i18n.t('challenge.update')}
    </RxButton>
  </RxFormItem>
</RxForm>
