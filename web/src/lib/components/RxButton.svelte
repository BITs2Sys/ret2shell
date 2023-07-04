<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  export let level: 'primary' | 'info' | 'success' | 'warning' | 'error' | null = null
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let ghost = false
  export let justify: 'start' | 'center' | 'end' = 'center'
  export let uppercase = false
  export let loading = false
  export let square = false
  export let disabled = false
  export let active = false
  export let type: 'button' | 'reset' | 'submit'| null = 'button'
  let clazz = ''
  export { clazz as class }

  const dispatch = createEventDispatcher()

  function handleClick() {
    dispatch('click')
  }
  /**
   * Possible classes:
   * btn text-base btn-ghost btn-xs btn-sm btn-md btn-lg
   * bg-base-content/5 backdrop-blur space-x-2
   * text-primary text-info text-success text-warning text-error
   * btn-primary btn-info btn-success btn-warning btn-error
   * btn-xl btn-2xl justify-start justify-center
   * justify-end normal-case btn-square
   */
  $: classes = [
    'btn',
    'border-none',
    'space-x-2',
    'text-base',
    'content-center',
    level === null && !ghost && 'bg-base-content/5 backdrop-blur',
    ghost && 'btn-ghost',
    level && `${ghost ? 'text' : 'btn'}-${level}`,
    active && 'text-primary',
    size && `btn-${size}`,
    justify && `justify-${justify}`,
    !uppercase && 'normal-case',
    square && 'btn-square',
    clazz,
  ]
    .filter(Boolean)
    .join(' ')
</script>

<button class={classes} on:click={handleClick} disabled={disabled || loading} {type} {...$$restProps}>
  {#if loading}
    <span class="loading" />
  {/if}
  <slot />
</button>
