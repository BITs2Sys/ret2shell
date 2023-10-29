<script lang="ts">
  import RxCard from '$lib/components/RxCard.svelte'
  import RxForm from '$lib/components/RxForm.svelte'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import Captcha from '$lib/blocks/Captcha.svelte'
  import { game, refreshTeam } from '$lib/stores/game'
  import { validator } from '@felte/validator-zod'
  import { createForm } from 'felte'
  import { z } from 'zod'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { createTeam } from '$lib/api/game'
  import type { AxiosError } from 'axios'
  import { showMessage } from '$lib/stores/toast'
  import { goto } from '$app/navigation'

  let schema = z.object({
    name: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('game.teamNameIsRequired') })
      .max(120, { message: $i18n.t('game.teamNameTooLong') }),
    captcha_id: z.string().trim(),
    captcha_answer: z
      .string()
      .trim()
      .min(1, { message: $i18n.t('account.captchaIsRequired') }),
  })
  let loading = false
  let captcha: Captcha | null
  const { form, data, touched, errors } = createForm({
    extend: validator({ schema }),
    onSubmit(values) {
      loading = true
      createTeam($game.current?.id || 0, values)
        .then(() => {
          goto(`/games/${$game.current?.id}`, { replaceState: true }).then(() => {
            showMessage('success', $i18n.t('game.createTeamSuccess'), 5000)
            refreshTeam()
          })
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('game.createTeamFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          captcha?.refreshAll()
        })
        .finally(() => {
          loading = false
        })
    },
  })
  const captchaAnswerValue = $data.captcha_answer
  $: {
    // console.log('answer', captchaAnswerValue, $data.captcha_answer)
    if (captchaAnswerValue !== $data.captcha_answer) {
      $touched.captcha_answer = true
    }
  }

  const captchaIdValue = $data.captcha_id
  $: {
    // console.log('id', captchaIdValue, $data.captcha_id)
    if (captchaIdValue !== $data.captcha_id) {
      $touched.captcha_id = true
    }
  }
</script>

<svelte:head>
  <title>{$i18n.t('game.createTeam')} - {$game.current?.name}</title>
</svelte:head>

<div class="flex-1 flex flex-col md:justify-center items-center p-3 lg:p-6">
  <RxCard class="flex flex-col md:flex-row w-full max-w-xl ">
    <div class="md:w-0 flex-1 flex-col">
      <h1 class="text-center font-bold text-base">{$i18n.t('game.createTeam')}</h1>
      <RxForm {form}>
        <RxFormItem
          name="name"
          label={$i18n.t('game.teamName')}
          hasError={$errors.name !== null}
          errors={$errors.name || ''}
        >
          <RxInput
            icon="icon-[fluent--flag-20-regular]"
            class="w-full"
            id="name"
            name="name"
            hasError={$errors.name !== null}
            autocomplete="name"
          />
        </RxFormItem>
        <Captcha
          bind:this={captcha}
          hasError={$errors.captcha_answer !== null}
          errors={$errors.captcha_answer || ''}
          bind:captchaId={$data.captcha_id}
          bind:captchaAnswer={$data.captcha_answer}
        />
        <RxFormItem name="submitAction" label="">
          <RxButton class="w-full" level="primary" type="submit" {loading}>
            {$i18n.t('game.createTeam')}
          </RxButton>
        </RxFormItem>
        <RxLink class="w-full" ghost href={`/games/${$game.current?.id}/join`}>
          {$i18n.t('game.joinTeamLink')}
        </RxLink>
      </RxForm>
    </div>
  </RxCard>
</div>
