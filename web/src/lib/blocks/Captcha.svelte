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
  import {encode} from 'js-base64'
  export let hasError = false
  export let errors: string | string[] | undefined = undefined
  export let captchaId = ''
  export let captchaAnswer = ''
  let loading = true
  let failed = false
  export let enabled = true
  let powing = true
  let fetchingImg = true
  let imgSrc = ''
  let captcha: Captcha

  function refresh() {
    getCaptcha().then((res) => {
      if (res.status !== 200) {
        failed = true
        loading = false
        return
      }
      res.json().then((data) => {
        captcha = data
        failed = false
        loading = false
        fetchingImg = false
      })
    })
  }

  onMount(() => {
    refresh()
  })
</script>

<RxFormItem label={$i18n.t('form.captcha')} name="captchaAnswer" class="" {hasError} {errors}>
  <input class="hidden" id="captcha_id" name="captcha_id" value={captchaId} />
  {#if loading || failed}
    <RxButton {loading} class="w-full">
      {loading ? $i18n.t('form.loadingCaptcha') : $i18n.t('form.reloadCaptcha')}
    </RxButton>
    <input id="captcha_answer" name="captcha_answer" class="hidden" value={captchaAnswer} />
  {:else if enabled && !loading && !failed && captcha.validator === Validator.Pow}
    <RxInput
      icon="icon-[fluent--beaker-16-regular]"
      class="w-full"
      id="captcha_answer"
      type="text"
      name="captcha_answer"
      {hasError}
      disabled
      value={captchaAnswer}
      placeholder="0XDEADBEEF######"
    >
      <RxButton loading={powing}>{$i18n.t('form.powing')}</RxButton>
    </RxInput>
  {:else if enabled && !loading && !failed && captcha.validator === Validator.Image}
    <RxInput
      icon="icon-[fluent--beaker-16-regular]"
      class="w-full"
      id="captcha_answer"
      type="text"
      name="captcha_answer"
      {hasError}
      value={captchaAnswer}
    >
      <RxButton class="border-none w-24 p-0 overflow-hidden" disabled={fetchingImg}>
        <RxImage class="w-full object-scale-down" loading={fetchingImg} src={`data:image/svg+xml;base64,${encode(captcha.challenge, false)}`} />
      </RxButton>
    </RxInput>
  {:else}
    <input id="captcha_answer" name="captcha_answer" class="hidden" value={captchaAnswer} />
  {/if}
</RxFormItem>
