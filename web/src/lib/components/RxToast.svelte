<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte'
  import { fly } from 'svelte/transition'
  import { quintOut } from 'svelte/easing'
  import { removeMessage } from '$lib/stores/toast'

  export let id: string
  export let message: string
  export let acceptMessage: string | undefined = undefined
  export let rejectMessage: string | undefined = undefined
  export let persistTime: number | undefined = undefined
  export let level: 'info' | 'warning' | 'success' | 'error' = 'info'

  $: {
    console.log(id, persistTime)
  }

  let progress: HTMLElement

  const dispatch = createEventDispatcher()

  function accept() {
    dispatch('accept')
    setTimeout(() => {
      removeMessage(id)
    }, 100)
  }

  function reject() {
    dispatch('reject')
    setTimeout(() => {
      removeMessage(id)
    }, 100)
  }

  onMount(() => {
    if (persistTime && persistTime > 0) {
      setTimeout(() => {
        progress.classList.remove('w-full')
        progress.classList.add('w-0')
      }, 100)
    }
  })

  $: progressClasses = [
    'w-full',
    'h-full',
    'border-b',
    'transition-all',
    'ease-linear',
    level === 'error' && 'border-b-error',
    level === 'warning' && 'border-b-warning',
    level === 'success' && 'border-b-success',
    level === 'info' && 'border-b-info',
  ]
    .filter(Boolean)
    .join(' ')
</script>

<div
  class="alert bg-neutral relative transition-all border-base-content/5"
  transition:fly={{ delay: 100, duration: 300, x: 0, y: 32, opacity: 0, easing: quintOut }}
>
  <div class="flex flex-row space-x-2 flex-1 items-center">
    {#if level === 'info'}
      <span class="icon-[fluent--info-16-regular] w-6 h-6 text-info" />
    {:else if level === 'warning'}
      <span class="icon-[fluent--warning-16-regular] w-6 h-6 text-warning" />
    {:else if level === 'success'}
      <span class="icon-[fluent--checkmark-circle-16-regular] w-6 h-6 text-success" />
    {:else if level === 'error'}
      <span class="icon-[fluent--dismiss-circle-16-regular] w-6 h-6 text-error" />
    {/if}
    <span class="text-base">{message}</span>
    <div class="flex-1" />
    {#if rejectMessage}
      <button class="btn btn-sm btn-ghost" on:click={reject}>{rejectMessage}</button>
    {/if}
    {#if acceptMessage}
      <button class="btn btn-sm btn-primary" on:click={accept}>{acceptMessage}</button>
    {/if}
  </div>
  {#if persistTime}
    <div class="absolute left-4 right-4 bottom-0 h-px">
      <div bind:this={progress} class={progressClasses} style={`transition-duration: ${persistTime}ms`} />
    </div>
  {/if}
</div>
