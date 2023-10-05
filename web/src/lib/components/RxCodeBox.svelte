<script lang="ts">
  import RxCodearea from './RxCodearea.svelte'

  let clazz = ''
  export { clazz as class }
  export let name: string | undefined = undefined
  export let disabled = false
  export let hasError = false
  export let placeholder = ''
  export let value = ''
  export let lang = 'markdown'
  export let loading = false
  export let readonly = false
  $: classes = [
    'textarea',
    'relative',
    'w-full',
    'h-[32rem]',
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
</script>

<div class={classes}>
  <RxCodearea class="bg-transparent" bind:value {name} {placeholder} {lang} {loading} {readonly} />
</div>
