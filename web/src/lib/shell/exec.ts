// This module provides a unique command execution solution for the shell.
import type { Command } from './commands/interface'
import * as commands from './commands'
import type { RnixStdio } from './stdio'
import type { ParseEntry } from 'shell-quote'
import type { RnixEnv } from './shell'

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
    // TODO
    return 0
  }
}
