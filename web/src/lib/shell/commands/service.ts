import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { RnixEnv } from '../shell'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import ansiColors from 'ansi-colors'

export class Service implements Command {
  name = 'service'
  man = get(i18n).t('shell.service.man')
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.logError(get(i18n).t('shell.noGameSpecified'))
      io.logInfo(get(i18n).t('shell.noGameSpecifiedTips'))
      return 1
    } else if (envp.challenge == null) {
      io.logError(get(i18n).t('shell.noChallengeSpecified'))
      io.logInfo(get(i18n).t('shell.noChallengeSpecifiedTips'))
      return 1
    }
    return 0
  }
}
