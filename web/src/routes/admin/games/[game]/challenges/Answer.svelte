<script lang="ts">
  import { createChallengeAnswer, getChallengeAnswer, updateChallengeAnswer } from '$lib/api/challenge'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCodeBox from '$lib/components/RxCodeBox.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Answer } from '$lib/models/answer'
  import type { Challenge } from '$lib/models/challenge'
  import { showMessage } from '$lib/stores/toast'
  import type { Obj } from '@felte/core'
  import { validator } from '@felte/validator-zod'
  import type { AxiosError } from 'axios'
  import { createForm } from 'felte'
  import { onMount } from 'svelte'
  import { z } from 'zod'

  export let challenge: Challenge
  let submitting: boolean = false
  let loading: boolean = false
  let answerExists: boolean | null = null
  let answer: Answer = {
    id: 0,
    title: '',
    published_at: 0,
    updated_at: 0,
    author_id: 0,
    challenge_id: 0,
    content: '',
  }
  let schema = z.object({
    title: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('answer.titleRequired') }),
    content: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('answer.contentRequired') }),
  })

  const { form, data, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      if (answerExists === null) return
      submitting = true
      const newAnswer: Answer = {
        ...answer,
        ...values,
      }
      console.log(answerExists)
      console.log(newAnswer)
      if (answerExists === true) {
        updateChallengeAnswer(challenge.id, newAnswer)
          .then(() => {
            showMessage('success', $i18n.t('answer.updateSuccess'), 5000)
            fetchAnswer()
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('answer.updateFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
          .finally(() => {
            submitting = false
          })
      } else if (answerExists === false) {
        createChallengeAnswer(challenge.id, newAnswer)
          .then(() => {
            showMessage('success', $i18n.t('answer.createSuccess'), 5000)
            fetchAnswer()
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('answer.createFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
          .finally(() => {
            submitting = false
          })
      }
    },
  })

  function fetchAnswer() {
    loading = true
    getChallengeAnswer(challenge.id)
      .then((res) => {
        answerExists = true
        answer = res
        data.update(() => {
          return res as unknown as Obj
        })
      })
      .catch(() => {
        showMessage('info', `${$i18n.t('answer.fetchFailed')}`, 5000)
        answerExists = false
      })
      .finally(() => {
        loading = false
      })
  }

  onMount(() => {
    fetchAnswer()
  })
</script>

<RxForm class="p-4 lg:p-6" {form}>
  <RxFormItem name="title" label={$i18n.t('answer.title')} hasError={$errors.title !== null} errors={$errors.title}>
    <RxInput
      icon="icon-[fluent--flag-16-regular]"
      class="w-full"
      id="title"
      name="title"
      disabled={submitting}
      hasError={$errors.title !== null}
      value={answer.title}
    />
  </RxFormItem>
  <RxFormItem
    name="content"
    label={$i18n.t('answer.content')}
    hasError={$errors.content !== null}
    errors={$errors.content}
  >
    <RxCodeBox
      name="content"
      class="h-[16rem]"
      lang="markdown"
      placeholder="Mode = Markdown"
      {loading}
      readonly={loading || submitting}
      value={answer.content}
      droppable={true}
    />
  </RxFormItem>
  <RxFormItem name="submitAction" label="">
    <RxButton class="w-full" type="submit" loading={submitting}>
      {submitting ? $i18n.t('answer.submiting') : $i18n.t('answer.submit')}
    </RxButton>
  </RxFormItem>
</RxForm>
