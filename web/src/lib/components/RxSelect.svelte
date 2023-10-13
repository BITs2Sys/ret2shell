<script lang="ts">
  import RxButton from './RxButton.svelte'
  import RxPopup from './RxPopup.svelte'
  import { createField } from 'felte'
  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' = 'md'
  export let availableOptions: { id: number | string | null; label: string }[] = []
  export let value: number | string | null = null
  export let disabled = false
  export let name: string
  const { field, onBlur, onInput } = createField(name)
</script>

<RxPopup
  {size}
  class="flex-1 justify-start bg-base-content/5 backdrop-blur border-none"
  {name}
  popupWidth="full"
  offset={6}
  {disabled}
>
  <span slot="button" use:field class="text-base flex flex-row items-center w-full">
    <span class="flex-1 text-start">
      {availableOptions.find((option) => option.id === value)?.label || 'Select'}
    </span>
    <span class="icon-[fluent--chevron-down-16-regular] w-5 h-5"></span>
  </span>
  <div class="rounded-box bg-neutral flex flex-col shadow-lg w-full p-2">
    {#each availableOptions as option}
      <RxButton
        ghost
        on:click={() => {
          value = option.id
          // console.log(option)
          onInput(option.id)
          onBlur()
        }}
        justify="start"
      >
        {option.label}
      </RxButton>
    {/each}
  </div>
</RxPopup>
