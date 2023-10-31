<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import RxButton from './RxButton.svelte'

  export let label: string | undefined = undefined
  export let preset: string | number | undefined = undefined
  export let value: unknown | string | number | undefined = undefined

  let clazz = ''
  export { clazz as class }
  $: classes = [
    'flex',
    'flex-row',
    'space-x-4',
    'justify-between',
    'outline-2',
    'outline-offset-2',
    'outline-base-content/40',
    value === preset && 'outline',
  ]
    .concat(clazz.split(' '))
    .filter(Boolean)
    .join(' ')

  const dispatch = createEventDispatcher()
</script>

<RxButton
  class={classes}
  on:click={() => {
    dispatch('select', preset)
  }}
>
  <slot>
    <span class="text-base">{label}</span>
  </slot>
  <span
    class={`icon-[fluent--checkmark-circle-20-regular] w-5 h-5 ${value === preset && 'text-primary'} flex-shrink-0`}
  />
</RxButton>
