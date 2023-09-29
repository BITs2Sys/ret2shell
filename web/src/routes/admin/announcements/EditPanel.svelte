<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCodearea from '$lib/components/RxCodearea.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { createEventDispatcher } from 'svelte'

  export let announcement: Announcement

  let clazz = ''
  export { clazz as class }

  $: classes = `absolute w-full bottom-0 flex flex-col overflow-hidden ${clazz}`

  const dispatch = createEventDispatcher()
</script>

<div class={classes}>
  <div
    class="h-16 min-h-16 border-y border-y-base-content/10 backdrop-blur bg-base-100/80 flex flex-row px-2 items-center space-x-2"
  >
    <div class="join flex-1">
      <RxButton ghost class="join-item">
        <span class={`icon-[fluent--pin-16-regular] w-5 h-5 ${announcement.pinned ? 'text-error' : 'opacity-60'}`}
        ></span>
      </RxButton>
      <RxInput ghost class="flex-1 join-item" label="Title" placeholder="Title" bind:value={announcement.title} />
    </div>
    <div class="join">
      <RxButton
        ghost
        level="primary"
        class="join-item"
        on:click={() => {
          dispatch('submit', announcement)
        }}>{$i18n.t('announcement.submit')}</RxButton
      >
      <RxButton
        ghost
        level="error"
        class="join-item ml-0"
        on:click={() => {
          dispatch('close')
        }}
      >
        <span class="icon-[fluent--dismiss-16-regular] w-5 h-5"></span>
      </RxButton>
    </div>
  </div>
  <RxCodearea class="flex-1" lang="markdown" readonly={false} bind:value={announcement.content} />
</div>
