<script lang="ts">
  import { popup, type PopupSettings, type Placement } from '$lib/utils/popup'

  export let name: string
  export let placement: Placement = 'bottom'
  export let event: 'click' | 'hover' | 'focus-blur' | 'focus-click' = 'click'
  export let offset = 16
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let disabled = false
  export let popupWidth: string | number = 48
  let clazz = ''
  export { clazz as class }

  // btn-xs btn-sm btn-md btn-lg btn-xl btn-2xl
  $: sizeClass = `btn-${size}`

  $: buttonClasses = ['btn', sizeClass, clazz].filter(Boolean).join(' ')
  // w-0 w-1 w-2 w-3 w-4 w-6 w-8 w-12 w-16 w-20 w-24 w-28 w-32 w-36 w-40 w-48 w-56 w-64 w-72 w-80 w-96 w-auto
  $: popupClasses = [`w-${popupWidth} flex flex-col space-y-2 z-50`].filter(Boolean).join(' ')

  const popupSettings: PopupSettings = {
    event: event,
    target: name,
    placement: placement,
    middleware: {
      offset: offset,
    },
  }
</script>

<button class={buttonClasses} use:popup={popupSettings} {disabled} type="button">
  <slot name="button">
    <span class="w-5 h-5 icon-[fluent--chevron-down-16-regular]" />
  </slot>
</button>
<div class={popupClasses} data-popup={name}>
  <slot />
</div>
