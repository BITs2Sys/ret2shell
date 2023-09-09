<script lang="ts">
  import RxRadioButton from './RxRadioButton.svelte'
  export let value: unknown | string | number | undefined = undefined
  export let direction: 'row' | 'column' = 'row'
  interface RxRadioItem {
    label: string
    value: string | number
  }
  export let items: RxRadioItem[]
  let clazz = ''
  export { clazz as class }

  $: classes = [
    'flex',
    'flex-wrap',
    direction === 'row' ? 'flex-row' : 'flex-col',
    direction === 'row' ? 'space-x-4' : 'space-y-4',
    clazz,
  ].join(' ')
</script>

<div class={classes}>
  {#each items as item}
    <RxRadioButton
      class="flex-1"
      {value}
      preset={item.value}
      label={item.label}
      on:select={(data) => {
          value = data.detail
      }}
    />
  {/each}
</div>
