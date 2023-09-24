import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { RnixEnv } from '../shell'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import ansiColors from 'ansi-colors'

export class Service implements Command {
  name = 'service'
  man = 'service'
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noGameSpecified'))}`)
      return 1
    } else if (envp.challenge == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noChallengeSpecified'))}`)
      return 1
    }
    return 0
  }
}
