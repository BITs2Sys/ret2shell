<script lang="ts">
  import RxButton from './RxButton.svelte'

  let clazz = ''
  export { clazz as class }
  export let hasError = false
  export let disabled = false
  export let type: 'text' | 'password' | 'number' = 'text'
  export let icon: string | undefined = undefined
  export let id: string | undefined = undefined
  export let name: string | undefined = undefined
  export let value: string | number | unknown | undefined = undefined

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

  $: classes = ['input', 'backdrop-blur', 'ml-0', 'bg-base-content/5', 'min-w-0', 'flex-1', hasError && 'input-error', 'join-item', clazz]
    .filter(Boolean)
    .join(' ')
</script>

{#if icon || type === 'password'}
  <div class="join flex-1">
    {#if icon}
      <span class="bg-base-content/20 join-item pl-4 pr-4 flex items-center">
        <div class={`w-5 h-5 ${icon}`} />
      </span>
    {/if}
    <input {id} {name} class={classes} {disabled} use:typeAction {...$$restProps} bind:value />
    {#if type === 'password'}
      <RxButton class="join-item ml-0" on:click={togglePasswordVisible}>
        <!-- icon-[fluent--eye-16-regular] and icon-[fluent--eye-off-16-regular] -->
        <div class={`w-5 h-5 icon-[fluent--${passwordVisible ? 'eye' : 'eye-off'}-16-regular]`} />
      </RxButton>
    {/if}
    <slot />
  </div>
{:else}
  <input {id} {name} class={classes.replace('join-item', '')} {disabled} use:typeAction {...$$restProps} bind:value />
{/if}
