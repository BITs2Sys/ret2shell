import type { Terminal } from 'xterm'
import ansiEscapes from 'isomorphic-ansi-escapes'

export class TerminalCursor {
  public x: number = 0
  public y: number = 0
  private readonly terminal: Terminal
  public constructor(term: Terminal) {
    this.terminal = term
  }

  public onData(f: (data: string) => void) {
    this.terminal.onData(f)
  }

  public syncFromTerminal() {
    const cursor = this.terminal.buffer.active.cursorY
    this.x = this.terminal.buffer.active.cursorX
    this.y = cursor < 0 ? 0 : cursor
  }

  public write(text: string | Uint8Array) {
    this.terminal.write(text, () => {
      this.syncFromTerminal()
    })
  }

  public writeAndBack(text: string | Uint8Array) {
    this.write(ansiEscapes.cursorSavePosition + text + ansiEscapes.cursorRestorePosition)
  }

  public cols() {
    return this.terminal.cols
  }
}
