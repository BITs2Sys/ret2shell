<script lang="ts">
  import { createEventDispatcher } from 'svelte'
  import RxButton from './RxButton.svelte'

  let clazz = ''
  export { clazz as class }
  export let hasError = false
  export let disabled = false
  export let size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
  export let type: 'text' | 'password' | 'number' = 'text'
  export let icon: string | undefined = undefined
  export let id: string | undefined = undefined
  export let name: string | undefined = undefined
  export let value: string | number | unknown | undefined = undefined
  export let ghost = false
  export let useFunc: (node: HTMLElement) => {
    destroy?(): void
  } = () => ({})

  let passwordVisible = false

  function togglePasswordVisible() {
    passwordVisible = !passwordVisible
    typeAction()
  }

  interface TypeAction {
    (node?: HTMLInputElement): void
    node?: HTMLInputElement
  }

  const typeAction: TypeAction = Object.assign(
    (node?: HTMLInputElement) => {
      if (node) {
        node.type = type == 'password' && passwordVisible ? 'text' : type

        typeAction.node = node as HTMLInputElement | undefined
      } else {
        if (typeAction.node) typeAction.node.type = type == 'password' && passwordVisible ? 'text' : type
      }
    },
    { node: (document.getElementById(id as string) as HTMLInputElement | null) || undefined }
  )

  $: classes = [
    'input',
    size === 'md' || size === 'lg' ? 'text-base' : 'text-sm',
    !ghost && 'backdrop-blur',
    'ml-0',
    ghost ? 'bg-transparent' : 'bg-base-content/5',
    'min-w-0',
    'flex-1',
    // input-xs input-sm input-md input-lg
    `input-${size}`,
    hasError && 'input-error',
    'join-item',
    clazz,
  ]
    .filter(Boolean)
    .join(' ')

  $: iconPad = size === 'xs' ? 'px-1' : size === 'sm' ? 'px-2' : size === 'md' ? 'px-4' : 'px-6'

  const dispatch = createEventDispatcher()
</script>

{#if icon || type === 'password'}
  <div class="join flex-1">
    {#if icon}
      <span class={`bg-base-content/20 join-item flex items-center ${iconPad}`}>
        <div class={`w-5 h-5 ${icon}`} />
      </span>
    {/if}
    <input
      {id}
      {name}
      class={classes}
      {disabled}
      use:typeAction
      {...$$restProps}
      bind:value
      on:blur={() => {
        dispatch('blur')
      }}
      use:useFunc
    />
    {#if type === 'password'}
      <RxButton class="join-item ml-0" on:click={togglePasswordVisible}>
        <!-- icon-[fluent--eye-16-regular] and icon-[fluent--eye-off-16-regular] -->
        <div class={`w-5 h-5 icon-[fluent--${passwordVisible ? 'eye' : 'eye-off'}-16-regular]`} />
      </RxButton>
    {/if}
    <slot />
  </div>
{:else}
  <input
    {id}
    {name}
    class={classes.replace('join-item', '')}
    {disabled}
    use:typeAction
    {...$$restProps}
    bind:value
    on:blur={() => {
      dispatch('blur')
    }}
    use:useFunc
  />
{/if}
