// This module provides shell entrypoint.
import { RnixStdio } from './stdio'
import { Exec } from './exec'
import type { Terminal } from 'xterm'
import ansiColors from 'ansi-colors'
import type { Challenge } from '$lib/models/challenge'
import type { Game } from '$lib/models/game'
import { BufferHistory } from './history'
import type { User } from '$lib/models/user'
import ansiEscapes from 'isomorphic-ansi-escapes'
import { parse } from 'shell-quote'
import stripAnsi from 'strip-ansi'

export interface RnixEnv {
  challenge: Challenge | null
  game: Game | null
  user: User | null
}

interface Prompt {
  head: string
}

export class RnixShell {
  private stdio: RnixStdio
  private exec: Exec
  private code = 0
  private env: RnixEnv = { challenge: null, game: null, user: null }
  private prompt: Prompt = { head: '$ ' }
  private history: BufferHistory
  private inputBuffer: string = ''
  private running: boolean = false

  public constructor(term: Terminal) {
    ansiColors.enabled = true
    this.history = new BufferHistory()
    this.stdio = new RnixStdio(term)
    this.exec = new Exec()
    this.buildPrompt()
  }

  public setChallenge(challenge: Challenge | null) {
    this.env.challenge = challenge
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public setGame(game: Game | null) {
    this.env.game = game
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public setUser(user: User | null) {
    this.env.user = user
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public async run() {
    if (this.running) return
    this.running = true
    let pendingBuffer = ''
    while (true) {
      this.stdio.print(this.prompt.head)
      while (true) {
        this.inputBuffer = await this.stdio.input(pendingBuffer)
        if (this.inputBuffer.endsWith(ansiEscapes.cursorUp1)) {
          this.stdio.clearInput()
          pendingBuffer = stripAnsi(this.history.previous(this.inputBuffer.trim()).trim())
          // console.log(pendingBuffer)
          continue
        } else if (this.inputBuffer.endsWith(ansiEscapes.cursorDown1)) {
          this.stdio.clearInput()
          pendingBuffer = stripAnsi(this.history.next().trim())
          // console.log(pendingBuffer)
          continue
        }
        pendingBuffer = ''
        break
      }
      this.stdio.print('\n')
      this.history.push(stripAnsi(this.inputBuffer))
      this.code = await this.exec.exec(this.stdio, parse(this.inputBuffer), this.env, this.inputBuffer)
    }
  }

  private buildPrompt() {
    this.prompt.head = `${ansiColors.green(this.env.user?.name || 'guest')}@${ansiColors.blue(
      this.env.game?.name || 'unknown'
    )} in ${ansiColors.yellow(this.env.challenge?.name || '/')} ${
      this.code === 0 ? '' : ansiColors.redBright('Error(' + this.code.toString() + ')')
    }\n${this.code === 0 ? ansiColors.greenBright.bold('$') : ansiColors.redBright.bold('$')} `
  }
}
