<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxCodearea from '$lib/components/RxCodearea.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
  import { i18n } from '$lib/i18n'
  import type { Announcement } from '$lib/models/announcement'
  import { createEventDispatcher } from 'svelte'

  export let announcement: Announcement
  export let loading = false
  export let submitting = false

  let clazz = ''
  export { clazz as class }

  $: classes = `absolute w-full bottom-0 flex flex-col overflow-hidden ${clazz}`

  const dispatch = createEventDispatcher()
</script>

<div class={classes}>
  <div
    class="h-16 min-h-16 border-b border-b-base-content/5 backdrop-blur bg-base-100 flex flex-row px-2 items-center space-x-2"
  >
    <div class="join flex-1">
      <RxButton
        on:click={() => {
          announcement.pinned = !announcement.pinned
        }}
        ghost
        {loading}
        class="join-item"
      >
        {#if !loading}
          <span class={`icon-[fluent--pin-16-regular] w-5 h-5 ${announcement.pinned ? 'text-error' : 'opacity-60'}`} />
        {/if}
      </RxButton>
      <RxInput
        ghost
        class="flex-1 join-item"
        label="Title"
        placeholder="Title"
        disabled={loading || submitting}
        bind:value={announcement.title}
      />
    </div>
    <div class="join">
      <RxButton
        ghost
        level="primary"
        disabled={loading || submitting}
        loading={submitting}
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
  <RxCodearea
    class="flex-1 bg-base-100/80 backdrop-blur"
    lang="markdown"
    {loading}
    readonly={loading || submitting}
    bind:value={announcement.content}
  />
</div>
