<script lang="ts">
  import { page } from '$app/stores'
  import { createEventDispatcher } from 'svelte'
  export let level: 'info' | 'success' | 'warning' | 'error' | null = null
  export let href: string
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let ghost = false
  export let justify: 'start' | 'center' | 'end' = 'center'
  export let uppercase = false
  export let square = false
  export let active = false
  export let disabled = false
  export let exactlyMatched = false
  let clazz = ''
  export { clazz as class }

  function startsWithPath(path: string) {
    let route = $page.url.pathname
      .replace($page.url.hash, '')
      .split('/')
      .map((p) => p.trim())
      .filter(Boolean)
    let link = path
      .replace($page.url.hash, '')
      .split('/')
      .map((p) => p.trim())
      .filter(Boolean)
    if (link.length > route.length) {
      return false
    }
    for (let i = 0; i < link.length; i++) {
      if (link[i] !== route[i]) {
        return false
      }
    }
    return true
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
    'btn no-animation',
    disabled && 'btn-disabled',
    'border-none',
    'space-x-2',
    'text-base',
    'content-center',
    level === null && !ghost && 'bg-base-content/5 backdrop-blur',
    ghost && 'btn-ghost',
    level && `${ghost ? 'text' : 'btn'}-${level}`,
    size && `btn-${size}`,
    justify && `justify-${justify}`,
    !uppercase && 'normal-case',
    square && 'btn-square',
    (active || (exactlyMatched ? $page.url.pathname === href : startsWithPath(href))) &&
      (ghost ? 'text-primary' : '!btn-primary'),
    clazz,
  ]
    .filter(Boolean)
    .join(' ')

  const dispatch = createEventDispatcher()
</script>

<a {href} class={classes} {...$$restProps} on:click={() => dispatch('click')}>
  <slot />
</a>
