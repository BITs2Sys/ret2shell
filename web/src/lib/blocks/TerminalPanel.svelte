<script lang="ts">
  import { theme } from '$lib/stores/theme'
  import { Terminal, type ITerminalOptions } from 'xterm'
  import { FitAddon } from 'xterm-addon-fit'
  import { WebLinksAddon } from 'xterm-addon-web-links'
  import { CanvasAddon } from 'xterm-addon-canvas'
  import { onDestroy, onMount } from 'svelte'
  import { RnixShell } from '$lib/shell/shell'
  import 'xterm/css/xterm.css'

  let clazz = ''
  export { clazz as class }
  $: classes = `flex-1 relative overflow-clip ${clazz}`
  let terminal: HTMLDivElement
  let shell: RnixShell | null = null

  const term = new Terminal({
    convertEol: true,
    allowTransparency: true,
    cursorBlink: true,
    cursorStyle: 'underline',
    drawBoldTextInBrightColors: false,
    theme: {
      foreground: $theme.colorScheme === 'dark' ? '#dddddd' : '#222222',
      background: '#00000000',
      cursor: '#0078D6',
      selectionBackground: '#88888840',
      blue: '#0078D6',
      yellow: '#FBBD23',
      green: '#36D399',
      red: '#F83030',
    },
    fontFamily: 'JetBrains Mono Regular, monospace',
    fontSize: 16,
    lineHeight: 1.2,
  } as ITerminalOptions)

  const fitAddon = new FitAddon()
  const weblinksAddon = new WebLinksAddon()
  const canvasAddon = new CanvasAddon()

  term.loadAddon(fitAddon)
  term.loadAddon(canvasAddon)
  term.loadAddon(weblinksAddon)

  onMount(() => {
    term.open(terminal)
    fitAddon.fit()
    term.focus()
    shell = new RnixShell(term)
    shell.run()

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit()
    })

    resizeObserver.observe(terminal)
  })

  onDestroy(() => {
    shell?.emulateCommand('exit')
  })
</script>

<div class={classes}>
  <div class="w-full h-full overflow-clip" bind:this={terminal} id="terminal"></div>
</div>
