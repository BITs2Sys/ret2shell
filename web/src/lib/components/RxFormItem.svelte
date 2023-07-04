<script lang="ts">
  export let name: string
  export let label: string
  export let hasError = false
  export let errors: string | string[] | undefined = undefined
  let clazz = ''
  export { clazz as class }

  $: classes = ['form-control', 'flex-1', 'min-w-0', clazz].filter(Boolean).join(' ')

  $: tooltipClasses = [
    hasError && 'tooltip',
    'tooltip-warning',
    'tooltip-bottom',
    'tooltip-open',
    'flex',
    'flex-row',
    'flex-1',
  ]
    .filter(Boolean)
    .join(' ')

  $: formattedErrors = Array.isArray(errors) ? errors.join('; ') : errors
</script>

<fieldset class={classes}>
  <label class="label" for={name}>
    <slot name="label">
      <span class="label-text opacity-60 font-bold">{label}</span>
    </slot>
  </label>
  <div class={tooltipClasses} data-tip={formattedErrors}>
    <slot />
  </div>
</fieldset>
