<script lang="ts">
  import { colorDefs } from '$lib/stores/theme'
  import { Terminal, type ITerminalOptions } from 'xterm'
  import { FitAddon } from 'xterm-addon-fit'
  import { WebLinksAddon } from 'xterm-addon-web-links'
  import { CanvasAddon } from 'xterm-addon-canvas'
  import { createEventDispatcher, onDestroy, onMount } from 'svelte'
  import { RnixShell } from '$lib/shell/shell'
  import 'xterm/css/xterm.css'
  import { goto } from '$app/navigation'
  import type { Game } from '$lib/models/game'
  import type { Challenge } from '$lib/models/challenge'
  import { userInfo } from '$lib/stores/user'

  let clazz = ''
  export { clazz as class }
  $: classes = `flex-1 relative overflow-hidden ${clazz}`
  let terminal: HTMLDivElement
  let shell: RnixShell | null = null
  export let game: Game | null
  export let challenge: Challenge | null
  export let availableChallenges: Challenge[]

  const dispatch = createEventDispatcher()

  const linkHandler = {
    activate(_event: MouseEvent, text: string) {
      if (text.startsWith('http://') || text.startsWith('https://')) {
        window.open(text, '_blank')
      } else if (text.startsWith('rnix-route://')) {
        const path = text.replace('rnix-route:/', '')
        goto(path)
      } else if (text.startsWith('rnix-cmd://')) {
        const action = text.replace('rnix-cmd://', '')
        // console.log('emulating', action)
        shell?.emulateCommand(action)
      } else if (text.startsWith('rnix-chal://')) {
        const challengeId = text.replace('rnix-chal://', '')
        window.location.hash = `#${challengeId}`

        // for WSRX Client
      } else if (text.startsWith('wsrx://')) {
        window.open(text, '_blank')
      }
    },
    allowNonHttpProtocols: true,
  }

  let colors = colorDefs()

  const term = new Terminal({
    convertEol: true,
    allowTransparency: true,
    cursorBlink: true,
    cursorStyle: 'underline',
    drawBoldTextInBrightColors: false,
    theme: {
      foreground: colors['base-content'],
      background: '#00000000',
      cursor: colors.primary,
      selectionBackground: '#88888840',
      blue: colors.info,
      yellow: colors.warning,
      green: colors.success,
      red: colors.error,
    },
    fontFamily: 'JetBrains Mono',
    fontSize: 16,
    lineHeight: 1.2,
    linkHandler: linkHandler,
    customGlyphs: true,
    letterSpacing: 0,
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
    shell = new RnixShell(term, dispatch)
    userInfo().then((user) => {
      // console.log('setting user', user?.name)
      shell?.setUser(user)
      shell?.run()
    })

    const resizeObserver = new ResizeObserver(() => {
      fitAddon.fit()
    })

    resizeObserver.observe(terminal)
  })

  onDestroy(() => {
    shell?.emulateCommand('exit')
  })

  $: {
    if (game) {
      shell?.setGame(game)
    } else {
      shell?.setGame(null)
    }
    if (challenge) {
      shell?.setChallenge(challenge)
    } else {
      shell?.setChallenge(null)
    }
    shell?.setAvailableChallenges(availableChallenges)
  }
</script>

<div class={classes}>
  <div class="w-full h-full overflow-clip" bind:this={terminal} id="terminal"></div>
</div>
