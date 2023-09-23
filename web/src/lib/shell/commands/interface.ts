import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { RnixEnv } from '../shell'

export interface Command {
  name: string
  func(io: RnixStdio, args: ParseEntry[], origin: string, envp: RnixEnv): Promise<number>
  man: string
}
