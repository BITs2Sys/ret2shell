import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import ansiColors from 'ansi-colors'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import type { RnixEnv } from '../shell'
import { goto } from '$app/navigation'

export class Cd implements Command {
  name = 'cd'
  man = get(i18n).t('shell.cd.man')
  func = async (io: RnixStdio, args: ParseEntry[], _origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noGameSpecified'))}`)
      return 1
    }
    if (args.length !== 1) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.cd.usage'))}`)
      return 1
    }
    const challenge_name = args[0]
    let challenge = envp.availableChallenges.find((challenge) => challenge.name === challenge_name)
    if (challenge) {
      window.location.hash = `#${challenge.id}`
    } else {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.cd.notFound'))}`)
    }
    return 0
  }
}
