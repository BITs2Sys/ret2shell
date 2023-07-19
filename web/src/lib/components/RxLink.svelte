<script lang="ts">
  import { page } from '$app/stores'
  export let href: string
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let ghost = false
  export let justify: 'start' | 'center' | 'end' = 'center'
  export let uppercase = false
  export let exactlyMatched = false
  let clazz = ''
  export { clazz as class }

  /**
   * Possible classes:
   * btn text-base btn-ghost btn-xs btn-sm btn-md btn-lg border-none
   * btn-xl btn-2xl justify-start justify-center
   * justify-end normal-case space-x-2 content-center
   */
  $: classes = [
    'btn',
    'border-none',
    'content-center',
    'text-base',
    ghost ? 'btn-ghost' : 'bg-base-content/5 backdrop-blur',
    size && `btn-${size}`,
    justify && `justify-${justify}`,
    !uppercase && 'normal-case',
    clazz,
  ]
    .filter(Boolean)
    .join(' ')
</script>

<a
  {href}
  class={classes}
  class:text-primary={exactlyMatched ? $page.route.id === href : $page.route.id?.startsWith(href)}
  {...$$restProps}
>
  <slot />
</a>
