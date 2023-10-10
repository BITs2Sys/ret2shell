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
import { i18n } from '$lib/i18n'
import { get } from 'svelte/store'
import type { EventDispatcher } from 'svelte'

export interface RnixEnv {
  availableChallenges: Challenge[]
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
  private env: RnixEnv = { availableChallenges: [], challenge: null, game: null, user: null }
  private prompt: Prompt = { head: '$ ' }
  private history: BufferHistory
  private inputBuffer: string = ''
  private running: boolean = false
  private dispatch: EventDispatcher<Record<string, any>> | null = null

  public constructor(term: Terminal, dispatch: EventDispatcher<Record<string, any>> | null = null) {
    ansiColors.enabled = true
    this.dispatch = dispatch
    this.history = new BufferHistory()
    this.stdio = new RnixStdio(term)
    this.exec = new Exec()
    this.buildPrompt()
  }

  public emulateCommand(command: string) {
    this.stdio.clearInput()
    this.stdio.emulateInput(command + '\n')
  }

  public greet() {
    this.stdio.println(ansiColors.blueBright(get(i18n).t('shell.welcome')))
    this.stdio.logInfo(
      get(i18n).t('shell.helpTips', { command: ansiEscapes.link(ansiColors.green('help'), 'rnix-cmd://help') })
    )
    this.stdio.println('')
  }

  public setChallenge(challenge: Challenge | null) {
    if (this.env.challenge?.id === challenge?.id) return
    this.env.challenge = challenge
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.greet()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public setAvailableChallenges(availableChallenges: Challenge[]) {
    if (this.env.availableChallenges === availableChallenges) return
    this.env.availableChallenges = availableChallenges
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.greet()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public setGame(game: Game | null) {
    if (this.env.game?.id === game?.id) return
    this.env.game = game
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.greet()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public setUser(user: User | null) {
    if (this.env.user?.id === user?.id) return
    this.env.user = user
    this.buildPrompt()
    if (this.running) {
      this.stdio.clearInput()
      this.stdio.clear()
      this.greet()
      this.stdio.print(this.prompt.head)
      this.inputBuffer = ''
    }
  }

  public async run() {
    if (this.running) return
    this.running = true
    this.greet()
    let pendingBuffer = ''
    // eslint-disable-next-line no-constant-condition
    while (true) {
      this.stdio.print(this.prompt.head)
      // eslint-disable-next-line no-constant-condition
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
      if (stripAnsi(this.inputBuffer).trim().length === 0) continue
      if (this.inputBuffer.trim() === 'exit') break
      this.history.push(stripAnsi(this.inputBuffer))
      const args = parse(this.inputBuffer)
      const result = await this.exec.exec(this.stdio, args, this.env, this.inputBuffer)
      if (this.dispatch) {
        this.dispatch('executed', {
          cmd: result.cmd,
          code: result.code,
        })
      }
      this.code = result.code
      this.stdio.print('\n')
      this.buildPrompt()
    }
  }

  private buildPrompt() {
    this.prompt.head = `${ansiColors.green(this.env.user?.name || 'guest')} ${ansiColors.dim('at')} ${ansiColors.blue(
      this.env.game?.name || 'unknown'
    )} ${ansiColors.dim('in')} ${ansiColors.yellow(this.env.challenge?.name || '/')} ${
      this.code === 0 ? '' : ansiColors.redBright('[' + this.code.toString() + ']')
    }\n${this.code === 0 ? ansiColors.greenBright.bold('$') : ansiColors.redBright.bold('$')} `
  }
}
