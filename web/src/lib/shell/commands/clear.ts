import type { RnixStdio } from '../stdio'
import type { Command } from './interface'

export class Clear implements Command {
  name = 'clear'
  man = 'clear'
  func = async (io: RnixStdio) => {
    io.clear()
    return 0
  }
}
