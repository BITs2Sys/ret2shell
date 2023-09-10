<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import { i18n } from '$lib/i18n'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  import { onMount } from 'svelte'
  import { getCaptcha } from '$lib/api/account'
  import type { Captcha } from '$lib/models/captcha'
  import { Validator } from '$lib/models/config'
  import { encode } from 'js-base64'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  export let hasError = false
  export let errors: string | string[] | undefined = undefined
  export let captchaId: string | unknown | undefined = undefined
  export let captchaAnswer: string | unknown | undefined = undefined
  let loading = true
  let failed = false
  export let enabled = true
  let powing = true
  let fetchingPow = true
  let fetchingImg = true
  let captcha: Captcha

  function refreshImg() {
    fetchingImg = true
    refresh()
  }

  export function refreshAll() {
    loading = true
    refresh()
  }

  function refreshPow() {
    fetchingPow = true
    refresh()
  }

  function refresh() {
    getCaptcha()
      .then((res) => {
        captcha = res
        captchaId = captcha.id
        captchaAnswer = ''
        failed = false
        loading = false
        fetchingImg = false
        fetchingPow = false
      })
      .catch((err) => {
        captchaId = ''
        captchaAnswer = ''
        failed = true
        loading = false
        fetchingImg = false
        fetchingPow = false
        showMessage('error', $i18n.t('form.fetchCaptchaFailed') + ': ' + (err as AxiosError).response?.data, 5000)
      })
  }

  $: {
    if (captcha && captcha.validator === Validator.Pow) {
      powing = true
      const worker = new Worker(new URL('$lib/workers/pow.worker.ts', import.meta.url), { type: 'module' })
      worker.postMessage({
        challenge: captcha.challenge,
      })
      worker.onmessage = (e) => {
        powing = false
        captchaAnswer = e.data
        worker.terminate()
      }
    }
  }

  onMount(() => {
    refresh()
  })
</script>

<RxFormItem label={$i18n.t('form.captcha')} name="captcha_answer" class="" {hasError} {errors}>
  {#if loading || failed}
    <RxButton {loading} class="w-full" disabled={loading} on:click={refreshAll}>
      {loading ? $i18n.t('form.loadingCaptcha') : $i18n.t('form.reloadCaptcha')}
    </RxButton>
  {:else if enabled && !loading && !failed && captcha.validator === Validator.Pow}
    <RxInput
      icon="icon-[fluent--beaker-16-regular]"
      class="w-full"
      type="text"
      {hasError}
      disabled
      bind:value={captchaAnswer}
      placeholder="0XDEADBEEF######"
    >
      <RxButton loading={powing} disabled={powing || fetchingPow} on:click={refreshPow}>
        {fetchingPow ? $i18n.t('form.fetchingPow') : powing ? $i18n.t('form.powing') : ''}
        {#if !fetchingPow && !powing}
          <span class="icon-[fluent--checkmark-16-regular] w-6 h-6 text-success" />
        {/if}
      </RxButton>
    </RxInput>
  {:else if enabled && !loading && !failed && captcha.validator === Validator.Image}
    <RxInput icon="icon-[fluent--beaker-16-regular]" class="w-full" type="text" {hasError} bind:value={captchaAnswer}>
      <RxButton
        class="border-none w-24 p-0 overflow-hidden join-item ml-0"
        disabled={fetchingImg}
        on:click={refreshImg}
      >
        <RxImage
          class="w-full object-scale-down"
          loading={fetchingImg}
          src={`data:image/svg+xml;base64,${encode(captcha.challenge, false)}`}
        />
      </RxButton>
    </RxInput>
  {/if}
</RxFormItem>
