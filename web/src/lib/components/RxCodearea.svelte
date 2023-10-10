<script lang="ts">
  import { theme } from '$lib/stores/theme'
  import ace from 'ace-builds'
  import 'ace-builds/esm-resolver'
  import '$lib/styles/codeeditor.scss'
  import { onDestroy, onMount } from 'svelte'
  import { blur } from 'svelte/transition'
  import { createField } from 'felte'
  import { i18n } from '$lib/i18n'
  import { writable } from 'svelte/store'
  import { nanoid } from 'nanoid'
  import { uploadMedia } from '$lib/api/media'
  import type { AxiosError } from 'axios'
  import { showMessage } from '$lib/stores/toast'
  import { getMediaPath } from '$lib/models/media'

  export let value: string
  export let lang: string = ''
  export let readonly: boolean = false
  export let placeholder: string = ''
  export let loading = false
  export let droppable = false
  let showDropTips = false
  let rendering = true
  let progress = writable(0)

  export let name: string = 'common_code_area'

  const { field, onBlur, onInput } = createField(name)

  let clazz = ''
  export { clazz as class }

  let editorElement: HTMLPreElement
  let editor: ace.Ace.Editor | null = null
  let contentBackup: string = ''

  $: watchValue(value)
  function watchValue(val: string) {
    if (contentBackup !== val && editor && typeof val === 'string') {
      editor.session.setValue(val)
      onInput(val)
      contentBackup = val
    }
  }

  $: watchReadonly(readonly)
  function watchReadonly(val: boolean) {
    if (editor) {
      editor.setReadOnly(val)
    }
  }

  $: classes = ['w-full', 'h-full', 'overflow-scroll', 'relative', clazz].join(' ')

  onMount(() => {
    setTimeout(() => {
      initEditor()
    })
  })

  function initEditor() {
    editor = ace.edit(editorElement, {
      mode: `ace/mode/${lang}`,
      theme: `ace/theme/${$theme.colorScheme === 'light' ? 'kuroir' : 'github_dark'}`,
      readOnly: readonly,
      showPrintMargin: false,
      highlightActiveLine: false,
      highlightGutterLine: false,
      showGutter: true,
      showLineNumbers: true,
      tabSize: 2,
      useSoftTabs: true,
      wrap: true,
      value,
      fontSize: 16,
      fontFamily: 'JetBrains Mono',
      cursorStyle: 'smooth',
      animatedScroll: true,
      fadeFoldWidgets: true,
      hScrollBarAlwaysVisible: false,
      scrollPastEnd: true,
      selectionStyle: 'text',
      placeholder,
      useWorker: false,
    })
    editor.container.style.lineHeight = '1.6'

    editor.on('change', function () {
      const content = editor?.getValue()
      value = content || ''
      onInput(value)
      contentBackup = value
    })

    editor.on('blur', function () {
      onBlur()
    })

    rendering = false
  }

  const unsubscribe = theme.subscribe((value) => {
    if (editor) editor.setTheme(`ace/theme/${value.colorScheme === 'light' ? 'kuroir' : 'github_dark'}`)
  })

  onDestroy(() => {
    unsubscribe()
    editor?.destroy()
  })

  const onUploadDragEnter = () => {
    showDropTips = true
  }

  const onUploadDragLeave = () => {
    showDropTips = false
  }

  const allowDrop = (event: Event) => {
    if (droppable) {
      event.preventDefault()
      event.stopPropagation()
    }
  }

  const drop = (event: Event) => {
    if (droppable) {
      event.preventDefault()
      event.stopPropagation()
      showDropTips = false
      handleDropUploadImage(event as DragEvent)
    }
  }

  function handleDropUploadImage(e: DragEvent) {
    const files = e.dataTransfer?.files
    if (files && files.length) {
      const file = files[0]
      const id = nanoid()
      editor?.insert(`\n[Uploading:${id}]\n`)
      uploadMedia(file, false, progress)
        .then((resp) => {
          const model = resp.model
          const path = getMediaPath(model)
          value = value.replace(`\n[Uploading:${id}]\n`, `\n![${model.name}](${path})\n`)
          if (resp.remaining === -1)
            showMessage('success', $i18n.t('form.imageUploadSuccess', { remaining: '∞' }), 5000)
          else showMessage('success', $i18n.t('form.imageUploadSuccess', { remaining: resp.remaining }), 5000)
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('form.imageUploadFail')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }
</script>

<div class={classes}>
  {#if droppable && showDropTips}
    <div class="alert alert-info shadow-lg rounded-lg absolute top-0 left-0 w-full z-10">
      <div class="flex flex-row items-center space-x-2">
        <span class="icon-[fluent--cloud-arrow-up-24-regular] w-6 h-6" />
        <span>{$i18n.t('form.uploadTips')}</span>
      </div>
    </div>
  {/if}
  {#if droppable && $progress && $progress !== 100 && $progress !== 0}
    <div class="alert alert-info shadow-lg rounded-lg absolute top-0 left-0 w-full z-10 flex flex-row">
      <div class="flex-1 flex flex-row items-center space-x-4">
        <span class="icon-[fluent--cloud-arrow-up-24-regular] w-6 h-6" />
        <progress class="progress flex-1" value={$progress} max="100"></progress>
        <span class="loading loading-spinner loading-sm"></span>
      </div>
    </div>
  {/if}
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 bg-base-100/80 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="absolute left-0 top-0 bottom-0 right-0 p-2"
    on:dragenter={onUploadDragEnter}
    on:dragleave={onUploadDragLeave}
    on:dragover={allowDrop}
    on:drop={drop}
  >
    <pre
      class={`w-full min-h-full relative bg-transparent opacity-80 ${rendering ? 'hidden' : ''}`}
      use:field
      bind:this={editorElement}></pre>
  </div>
</div>
