<script lang="ts">
  import { uploadMedia } from '$lib/api/v1/media'
  import { i18n } from '$lib/i18n'
  import { getMediaPath } from '$lib/models/media'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { writable } from 'svelte/store'
  import RxButton from './RxButton.svelte'
  import { createField } from 'felte'

  export let value: string
  export let name: string
  let clazz = ''
  export { clazz as class }

  $: classes = `flex-1 aspect-video relative rounded-box bg-base-content/10 overflow-hidden ${clazz}`

  let inputEl: HTMLInputElement
  const progress = writable(0)

  const { field, onBlur, onInput } = createField(name)

  function handleUpload() {
    if (inputEl.files && inputEl.files.length > 0) {
      const file = inputEl.files[0]
      uploadMedia(file, false, progress)
        .then((resp) => {
          value = getMediaPath(resp.model)
          onInput(value)
          onBlur()
          if (resp.remaining === -1)
            showMessage('success', $i18n.t('form.imageUploadSuccess', { remaining: '∞' }), 5000)
          else showMessage('success', $i18n.t('form.imageUploadSuccess', { remaining: resp.remaining }), 5000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('form.imageUploadFail')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }
</script>

<input
  type="file"
  class="hidden"
  bind:this={inputEl}
  on:change={() => {
    handleUpload()
  }}
/>
<div class={classes} use:field>
  {#if value && value.length > 0}
    <img src={value} alt="" class="absolute inset-0 top-0 left-0 w-full h-full object-cover" />
  {:else}
    <p class="absolute top-0 left-0 w-full h-full flex flex-row items-center justify-center space-x-2">
      <span class="icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5"></span>
      <span class="font-bold text-base">{$i18n.t('form.upload')}</span>
    </p>
  {/if}
  <RxButton
    class="absolute inset-0 w-full h-full opacity-0 hover:opacity-80 hover:backdrop-blur transition-all"
    on:click={() => {
      inputEl.click()
    }}
  >
    <div class="flex flex-row items-center space-x-2">
      <span class="icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5"></span>
      <span class="font-bold text-base">{$i18n.t('form.upload')}</span>
    </div>
  </RxButton>
  {#if $progress && $progress !== 100 && $progress !== 0}
    <div class="alert !w-auto shadow-lg rounded-lg absolute top-4 left-4 right-4 z-10 flex flex-row">
      <div class="flex-1 flex flex-row items-center space-x-4">
        <span class="icon-[fluent--cloud-arrow-up-20-regular] w-5 h-5" />
        <progress class="progress flex-1" value={$progress} max="100"></progress>
        <span class="loading loading-spinner loading-sm"></span>
      </div>
    </div>
  {/if}
</div>
