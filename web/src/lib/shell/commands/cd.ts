import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import type { RnixEnv } from '../shell'

export class Cd implements Command {
  name = 'cd'
  man = get(i18n).t('shell.cd.man')
  func = async (io: RnixStdio, args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.logError(get(i18n).t('shell.noGameSpecified'))
      io.logInfo(get(i18n).t('shell.noGameSpecifiedTips'))
      return 1
    }
    if (origin.replace(this.name, '').trim().length <= 0) {
      io.logError(get(i18n).t('shell.cd.needChallengeName'))
      io.logInfo(get(i18n).t('shell.cd.usage'))
      return 1
    }
    const challenge_name = origin.replace(this.name, '').trim()
    let challenge = envp.availableChallenges.find((challenge) => challenge.name === challenge_name)
    if (challenge) {
      window.location.hash = `#${challenge.id}`
    } else {
      io.logError(get(i18n).t('shell.cd.notFound'))
    }
    return 0
  }
}
