// This module provides a unique command execution solution for the shell.
import type { Command } from './commands/interface'
import * as commands from './commands'
import type { RnixStdio } from './stdio'
import type { ParseEntry } from 'shell-quote'
import type { RnixEnv } from './shell'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'

export class Exec {
  commands: Map<string, Command>

  public constructor() {
    this.commands = new Map()
    for (const command of Object.values(commands)) {
      const cmd = new command()
      this.commands.set(cmd.name, cmd)
    }
  }

  public async exec(io: RnixStdio, args: ParseEntry[], env: RnixEnv, origin: string) {
    let cmd = args[0]
    if (typeof cmd !== 'string') {
      io.logError(get(i18n).t('shell.commandInvalid'))
      return { cmd: '', code: -127 }
    }
    cmd = cmd.trim()
    if (cmd === '') return { cmd, code: 0 }
    if (this.commands.has(cmd)) {
      return { cmd, code: await this.commands.get(cmd)!.func(io, args.slice(1), origin, env) }
    } else {
      io.logError(get(i18n).t('shell.commandNotFound', { command: cmd }))
      return { cmd, code: -127 }
    }
  }
}
