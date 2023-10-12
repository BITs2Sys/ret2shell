import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { RnixEnv } from '../shell'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import { game } from '$lib/stores/game'
import ansiEscapes from 'isomorphic-ansi-escapes'
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

    // if (get(game).runningInstance && get(game).runningInstance?.challenge_id !== envp.challenge.id) {
    io.logWarning(get(i18n).t('shell.service.alreadyRunningAnotherInstance'))
    io.print(
      `${get(i18n).t('shell.service.alreadyRunningAnotherInstanceConfirm')} [${ansiEscapes.link(
        ansiColors.greenBright('Yes'),
        'rnix-cmd://Yes'
      )}/${ansiEscapes.link(ansiColors.redBright('No'), 'rnix-cmd://No')}]: `
    )
    const choice = await io.input()
    io.println('')
    if (choice.toLowerCase().startsWith('y')) {
      io.logInfo(get(i18n).t('shell.service.willStoppingPreviousInstance'))
    } else {
      io.logInfo(get(i18n).t('shell.service.willNotStoppingPreviousInstance'))
      return 0
    }
    // }
    return 0
  }
}
