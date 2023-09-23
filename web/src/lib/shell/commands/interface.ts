export interface Command {
  name: string
  func(args: string[], origin: string, envp: string[]): Promise<number>
  man: string
}
