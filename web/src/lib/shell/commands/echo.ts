import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'

export class Echo implements Command {
  name = 'echo'
  man = 'echo'
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string) => {
    io.println(origin.replace('echo ', '').trim())
    return 0
  }
}
