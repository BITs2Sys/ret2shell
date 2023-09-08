<script lang="ts">
    import RxButton from "./RxButton.svelte"

  let clazz = ''
  export { clazz as class }
  $: classes = ['flex', 'flex-col', 'rounded-box', 'bg-base-content/5', 'backdrop-blur', 'shadow', clazz]
    .filter(Boolean)
    .join(' ')

  export let year = new Date().getFullYear()
  export let month = new Date().getMonth() + 1
</script>

<div class={classes}>
  <div class="h-16 flex flex-row items-center justify-between p-2 border-b border-b-base-content/5">
    <div class="flex flex-row space-x-2">
        <RxButton ghost>
            <span class="icon-[fluent--chevron-double-left-16-regular] w-6 h-6" />
        </RxButton>
        <RxButton ghost>
            <span class="icon-[fluent--chevron-left-16-regular] w-6 h-6" />
        </RxButton>
    </div>
    <span class="font-bold">{year} - {month.toString().padStart(2, '0')}</span>
    <div class="flex flex-row space-x-2">
        <RxButton ghost>
            <span class="icon-[fluent--chevron-right-16-regular] w-6 h-6" />
        </RxButton>
        <RxButton ghost>
            <span class="icon-[fluent--chevron-double-right-16-regular] w-6 h-6" />
        </RxButton>
    </div>
  </div>
</div>
