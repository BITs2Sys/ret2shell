import type { Command } from './interface'

export class Example implements Command {
  name = 'example-cmd'
  man = 'example'
  func = async (args: string[], origin: string, envp: string[]) => {
    return 0
  }
}
