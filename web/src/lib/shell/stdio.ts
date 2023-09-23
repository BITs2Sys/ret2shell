import type { Terminal } from 'xterm'
import { TerminalCursor } from './cursor'
import ansiEscapes from 'isomorphic-ansi-escapes'
import stringWidth from './utils/unicode'
import { offsetToColRow } from './utils/pos'
import ansiColors from 'ansi-colors'

export class RnixStdio {
  private userBuffer: string = ''
  private termCursor: TerminalCursor
  private inputCursor: number = 0
  private activeRead: boolean = false
  private displayBuffer: string = ''
  private resolve: (value: string) => void = () => {}
  private bufferLock: boolean = false
  private prefixX: number = 0

  /// init with a xterm terminal instance.
  /// should be only init once per terminal instance.
  public constructor(term: Terminal) {
    this.termCursor = new TerminalCursor(term)
    this.termCursor.onData((data) => {
      this.emulateInput(data)
    })
  }

  /// print text to terminal.
  public print(text: string) {
    this.displayBuffer += text
    this.flush()
  }

  /// print text to terminal, a newline will be appended.
  public println(text: string) {
    this.print(text + '\n')
  }

  public clearInput() {
    this.setInputCursor(0)
    this.userBuffer = ''
    this.displayBuffer += ansiEscapes.eraseDown
    this.flush()
  }

  /// clear the terminal.
  public clear() {
    this.displayBuffer += ansiEscapes.clearScreen
    this.flush()
  }

  /// emulate user input to terminal.
  public emulateInput(data: string) {
    if (!this.activeRead) return
    if (data.length > 2 && data.charCodeAt(0) !== 0x1b) {
      const _data = data.replace(/[\r\n]+/g, '\r') // the `Enter` key just sends `\r`
      for (const c of Array.from(_data)) {
        // console.log(c)
        this.writeInput(c)
      }
    } else {
      this.writeInput(data)
    }
  }

  /// read user input from terminal with custom prompt, function act as python `input`.
  public input(pendingBuffer: string): Promise<string> {
    if (this.bufferLock) return Promise.reject('buffer locked')
    this.bufferLock = true
    return new Promise((resolve) => {
      this.resolve = resolve
      this.userBuffer = ''
      this.inputCursor = 0
      this.termCursor.syncFromTerminal()
      this.prefixX = this.termCursor.x
      this.flush()
      this.activeRead = true
      if (pendingBuffer.length > 0) this.emulateInput(pendingBuffer)
    })
  }

  private flush() {
    this.termCursor.write(this.displayBuffer)
    this.displayBuffer = ''
  }

  private writeInput(data: string) {
    const ord = data.charAt(0)
    if (ord === ansiEscapes.ESC) {
      switch (data) {
        case ansiEscapes.cursorUp1:
        case ansiEscapes.cursorDown1:
          this.userBuffer += data
          this.finishInput()
          break
        case ansiEscapes.cursorBackward1:
          this.moveInputCursor(-1)
          break
        case ansiEscapes.cursorForward1:
          this.moveInputCursor(1)
          break
        case ansiEscapes.home:
          this.setInputCursor(0)
          break
        case ansiEscapes.end:
          this.setInputCursor(this.userBuffer.length)
          break
      }
    } else if (ord.charCodeAt(0) < 32 || ord.charCodeAt(0) === 0x7f) {
      switch (data) {
        case '\r': // enter
          this.finishInput()
          break
        case '\x01': // ctrl-a
          this.setInputCursor(0)
          break
        case '\x7f': // backspace
          if (this.inputCursor > 0) {
            // console.log('backspace: ', this.inputCursor)
            this.moveInputCursor(-1)
            this.userBuffer = this.userBuffer.slice(0, this.inputCursor) + this.userBuffer.slice(this.inputCursor + 1)
            this.reprintInput()
          }
          break
        case '\x03': // ctrl-c
          this.userBuffer = ''
          this.println(ansiColors.bold.bgBlackBright.white('^C'))
          this.finishInput()
          break
      }
    } else {
      this.insertInput(data)
    }

    this.flush()
  }

  private finishInput() {
    this.activeRead = false
    this.bufferLock = false
    this.resolve(this.userBuffer)
  }

  private getTermCursorOffset(input: string, offset: number): number {
    const newInput = input.slice(0, offset)
    return stringWidth(newInput) + this.prefixX
  }

  private moveInputCursor(offset: number) {
    const newCursor = this.inputCursor + offset
    this.setInputCursor(newCursor)
  }

  private setInputCursor(newCursor: number) {
    if (newCursor < 0 || newCursor > this.userBuffer.length) return
    const inputWithPrompts = '0'.repeat(this.prefixX) + this.userBuffer
    const prevOffset = this.getTermCursorOffset(this.userBuffer, this.inputCursor)
    this.inputCursor = newCursor
    const newOffset = this.getTermCursorOffset(this.userBuffer, newCursor)

    // console.log('cols: ', this.termCursor.cols())

    const { col: prevCol, row: prevRow } = offsetToColRow(inputWithPrompts, prevOffset, this.termCursor.cols())
    const { col: newCol, row: newRow } = offsetToColRow(inputWithPrompts, newOffset, this.termCursor.cols())

    this.displayBuffer += ansiEscapes.cursorMove(newCol - prevCol, newRow - prevRow)
    // console.log(`cursor move: ${prevCol} -> ${newCol}, ${prevRow} -> ${newRow}`)
  }

  private reprintInput() {
    this.displayBuffer +=
      ansiEscapes.eraseDown +
      ansiEscapes.cursorSavePosition +
      this.userBuffer.slice(this.inputCursor) +
      ansiEscapes.cursorRestorePosition
  }

  private insertInput(text: string) {
    // console.log(`cursor pos ${this.termCursor.x} ${this.termCursor.y}`)
    this.displayBuffer +=
      text +
      ansiEscapes.cursorSavePosition +
      this.userBuffer.slice(this.inputCursor) +
      ansiEscapes.cursorRestorePosition
    if (this.termCursor.x + stringWidth(text) === this.termCursor.cols()) {
      this.displayBuffer += ansiEscapes.cursorNextLine
    }
    this.userBuffer = this.userBuffer.slice(0, this.inputCursor) + text + this.userBuffer.slice(this.inputCursor)
    this.inputCursor += text.length
  }
}
