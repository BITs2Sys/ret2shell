<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  export let level: 'primary' | 'info' | 'success' | 'warning' | 'error' | null = null
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let ghost = false
  export let bold = true
  export let justify: 'start' | 'center' | 'end' = 'center'
  export let uppercase = false
  export let loading = false
  export let disabled = false
  export let square = false
  export let active = false
  export let type: 'button' | 'reset' | 'submit' | null = 'button'
  let clazz = ''
  export { clazz as class }
  let innerButton: HTMLButtonElement

  const dispatch = createEventDispatcher()

  export function tryScrollInToView() {
    innerButton.scrollIntoView({ behavior: 'smooth' })
  }

  function handleClick() {
    dispatch('click')
  }
  /**
   * Possible classes:
   * btn text-base btn-ghost btn-xs btn-sm btn-md btn-lg btn-xl btn-2xl
   * bg-base-content/5 backdrop-blur space-x-2
   * text-primary text-info text-success text-warning text-error
   * btn-primary btn-info btn-success btn-warning btn-error
   * btn-xl btn-2xl justify-start justify-center
   * justify-end normal-case btn-square
   */
  $: classes = [
    'btn no-animation',
    'flex flex-row flex-nowrap items-center',
    'border-none',
    'space-x-2',
    'text-base',
    'content-center',
    level === null && !ghost && 'bg-base-content/5 backdrop-blur',
    ghost && 'btn-ghost',
    level && `${ghost ? 'text' : 'btn'}-${level}`,
    active && (ghost ? 'text-primary' : '!btn-primary'),
    size && `btn-${size}`,
    bold ? 'font-bold' : 'font-normal',
    justify && `justify-${justify}`,
    !uppercase && 'normal-case',
    square && 'btn-square',
    clazz,
  ]
    .filter(Boolean)
    .join(' ')
</script>

<button
  class={classes}
  on:click={handleClick}
  disabled={disabled || loading}
  {type}
  bind:this={innerButton}
  {...$$restProps}
>
  {#if loading}
    <span class="loading loading-spinner loading-sm" />
  {/if}
  <slot />
</button>
