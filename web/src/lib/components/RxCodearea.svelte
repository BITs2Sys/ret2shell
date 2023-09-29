<script lang="ts">
  import { theme } from '$lib/stores/theme'
  import ace from 'ace-builds'
  import 'ace-builds/esm-resolver'
  import '$lib/styles/codeeditor.scss'
  import { onDestroy, onMount } from 'svelte'
  export let value: string
  export let lang: string = 'markdown'
  export let readonly: boolean = false
  let clazz = ''
  export { clazz as class }

  let editorElement: HTMLPreElement
  let editor: ace.Ace.Editor | null = null
  let contentBackup: string = ''

  $: watchValue(value)
  function watchValue(val: string) {
    if (contentBackup !== val && editor && typeof val === 'string') {
      editor.session.setValue(val)
      contentBackup = val
    }
  }

  $: classes = ['w-full', 'h-full', 'overflow-scroll', 'bg-base-100/80', 'backdrop-blur', 'relative', '', clazz].join(
    ' '
  )

  onMount(() => {
    initEditor()
  })

  function initEditor() {
    editor = ace.edit(editorElement, {
      mode: `ace/mode/${lang}`,
      theme: `ace/theme/${$theme.colorScheme === 'light' ? 'github' : 'twilight'}`,
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
      hScrollBarAlwaysVisible: true,
      scrollPastEnd: true,
      selectionStyle: 'text',
    })
    editor.container.style.lineHeight = '1.6'

    editor.on('change', function () {
      const content = editor?.getValue()
      value = content || ''
      contentBackup = value
    })
  }

  const unsubscribe = theme.subscribe((value) => {
    if (editor) editor.setTheme(`ace/theme/${value.colorScheme === 'light' ? 'github' : 'twilight'}`)
  })

  onDestroy(() => {
    unsubscribe()
    editor?.destroy()
  })
</script>

<div class={classes}>
  <div class="absolute left-0 top-0 bottom-0 right-0 p-4">
    <pre class="w-full min-h-full relative bg-transparent" bind:this={editorElement}></pre>
  </div>
</div>
