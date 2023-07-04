<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import RxImage from '$lib/components/RxImage.svelte'
  import { i18n } from '$lib/i18n'
  import RxFormItem from '$lib/components/RxFormItem.svelte'
  export let hasError = false
  export let errors: string | string[] | undefined = undefined
  export let captchaId = ''
  export let captchaAnswer = ''
  let loading = true
  let failed = false
  export let enabled = true
  let type: 'pow' | 'image' = 'pow'
  let powing = true
  let fetchingImg = true
  let imgSrc = ''
</script>

<RxFormItem label={$i18n.t('form.captcha')} name="captchaAnswer" class="" {hasError} {errors}>
  <input class="hidden" name="captchaId" value={captchaId} />
  {#if loading || failed}
    <RxButton {loading} class="w-full">
      {loading ? $i18n.t('form.loadingCaptcha') : $i18n.t('form.reloadCaptcha')}
    </RxButton>
    <input name="captchaAnswer" class="hidden" value={captchaAnswer} />
  {:else if enabled && !loading && !failed && type === 'pow'}
    <RxInput
      icon="icon-[fluent--beaker-16-regular]"
      class="w-full"
      id="captchaAnswer"
      type="text"
      name="captchaAnswer"
      {hasError}
      disabled
      value={captchaAnswer}
      placeholder="0XDEADBEEF######"
    >
      <RxButton loading={powing}>{$i18n.t('form.powing')}</RxButton>
    </RxInput>
  {:else if enabled && !loading && !failed && type === 'image'}
    <RxInput
      icon="icon-[fluent--beaker-16-regular]"
      class="w-full"
      id="captchaAnswer"
      type="text"
      name="captchaAnswer"
      {hasError}
      value={captchaAnswer}
    >
      <RxButton class="border-none w-24 p-0 overflow-hidden" disabled={fetchingImg}>
        <RxImage class="w-full h-full" loading={fetchingImg} src={imgSrc} />
      </RxButton>
    </RxInput>
  {:else}
    <input name="captchaAnswer" class="hidden" value={captchaAnswer} />
  {/if}
</RxFormItem>
