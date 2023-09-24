import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import { i18n } from '$lib/i18n'
import { get } from 'svelte/store'
import { AxiosError } from 'axios'
import type { RnixEnv } from '../shell'
import { downloadChallengeAttachment } from '$lib/api/challenge'
import ansiColors from 'ansi-colors'

export class Wget implements Command {
  name = 'wget'
  man = get(i18n).t('shell.wget.man')
  func = async (io: RnixStdio, args: ParseEntry[], _origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noGameSpecified'))}`)
      return 1
    } else if (envp.challenge == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noChallengeSpecified'))}`)
      return 1
    }
    if (args.length != 1) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.wget.usage'))}`)
      return 1
    }
    let file = args[0].toString().trim()

    try {
      await downloadChallengeAttachment(envp.game?.id as number, envp.challenge?.id as number, file, (progress) => {
        io.print(
          `\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b` +
            `${get(i18n).t('shell.wget.downloading')}: [${'='.repeat(Math.ceil(progress / 5)).padEnd(20)}] ${progress}%`
        )
      })
    } catch (err: unknown) {
      if (err instanceof AxiosError) {
        io.println(`${get(i18n).t('shell.wget.failed')}: ${err.response?.data}`)
        return 255
      } else {
        io.println(`${get(i18n).t('shell.wget.failed')}: ${err}`)
        return 255
      }
    }
    io.println(`\n\n${get(i18n).t('shell.wget.success')}`)
    return 0
  }
}
