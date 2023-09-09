<script lang="ts">
  import { theme } from '$lib/stores/theme'
  import { i18n } from '$lib/i18n'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { createEventDispatcher } from 'svelte'
  let clazz = ''
  export { clazz as class }
  export let id: string | undefined = undefined
  export let name: string | undefined = undefined
  export let droppable = false
  let showDropTips = false
  export let disabled = false
  export let hasError = false
  export let placeholder = ''
  export let progress = 0
  export let value = ''
  const dispatch = createEventDispatcher()
  let textarea: HTMLTextAreaElement
  $: classes = [
    'textarea',
    'relative',
    'w-full',
    'max-h-[32rem]',
    'flex-1',
    'focus-within:outline',
    'outline-2',
    disabled && 'bg-base-200 cursor-not-allowed',
    'outline-offset-2',
    hasError ? 'outline-error textarea-error' : 'outline-base-content/20',
    'backdrop-blur',
    'bg-base-content/5',
    clazz,
  ].join(' ')

  $: textareaClasses = [
    'w-full',
    'text-base',
    'leading-8',
    'bg-transparent',
    'outline-none',
    disabled && 'bg-base-200 cursor-not-allowed',
    'focus:outline-none',
    'resize-none',
    'overflow-hidden',
  ]
    .filter(Boolean)
    .join(' ')

  function onUploadDragEnter() {
    showDropTips = true
  }
  function onUploadDragLeave() {
    showDropTips = false
  }
  function allowDrop(event: Event) {
    if (droppable) {
      event.preventDefault()
      event.stopPropagation()
    }
  }
  function drop(event: Event) {
    if (droppable) {
      event.preventDefault()
      event.stopPropagation()
      showDropTips = false
      if (textarea) {
        dispatch('drop', { event, dropPosition: textarea.selectionStart })
      }
    }
  }

  function handleBlur() {
    dispatch('blur')
  }

  function refreshHeight(_value: string) {
    if (textarea) {
      if ('style' in textarea) {
        textarea.style.height = 'auto'
        textarea.style.height = textarea.scrollHeight + 'px'
      }
    }
  }

  $: {
    refreshHeight(value)
  }
</script>

<OverlayScrollbarsComponent
  class={classes}
  {...$$restProps}
  options={{
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-light' : 'os-theme-dark', autoHide: 'scroll' },
  }}
  defer
>
  {#if droppable && showDropTips}
    <div class="alert alert-info shadow-lg rounded-lg sticky top-0 left-0 w-full">
      <div>
        <cloud-arrow-up24-regular class="w-6 h-6" />
        <span>{$i18n.t('component.uploadTips')}</span>
      </div>
    </div>
  {/if}
  {#if droppable && progress && progress !== 100 && progress !== 0}
    <div class="alert shadow-lg rounded-lg sticky top-0 left-0 w-full">
      <div class="w-full">
        <cloud-arrow-up24-regular class="w-6 h-6" />
        <progress class="progress progress-info w-full" value={progress} max="100" />
      </div>
    </div>
  {/if}

  <textarea
  {id} {name}
    bind:this={textarea}
    class={textareaClasses}
    {placeholder}
    {disabled}
    on:dragenter={onUploadDragEnter}
    on:dragleave={onUploadDragLeave}
    on:dragover={allowDrop}
    on:drop={drop}
    on:blur={handleBlur}
    bind:value
  />
</OverlayScrollbarsComponent>
