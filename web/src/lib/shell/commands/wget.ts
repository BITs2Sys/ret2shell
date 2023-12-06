import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import { i18n } from '$lib/i18n'
import { get } from 'svelte/store'
import { AxiosError } from 'axios'
import type { RnixEnv } from '../shell'
import { downloadChallengeAttachment } from '$lib/api/v1/challenge'
import ansiEscapes from 'isomorphic-ansi-escapes'

export class Wget implements Command {
  name = 'wget'
  man = get(i18n).t('shell.wget.man')
  func = async (io: RnixStdio, args: ParseEntry[], _origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.logError(get(i18n).t('shell.noGameSpecified'))
      io.logInfo(get(i18n).t('shell.noGameSpecifiedTips'))
      return 1
    } else if (envp.challenge == null) {
      io.logError(get(i18n).t('shell.noChallengeSpecified'))
      io.logInfo(get(i18n).t('shell.noChallengeSpecifiedTips'))
      return 1
    }
    if (args.length != 1) {
      io.logError(get(i18n).t('shell.wget.usage'))
      return 1
    }
    const file = args[0].toString().trim()

    try {
      await downloadChallengeAttachment(envp.game?.id as number, envp.challenge?.id as number, file, (progress) => {
        io.print(
          ansiEscapes.cursorMove(0) +
            `${get(i18n).t('shell.wget.downloading')}: [${'='.repeat(Math.ceil(progress / 4)).padEnd(25)}] ${progress
              .toString()
              .padStart(3)}%`
        )
      })
    } catch (err: unknown) {
      if (err instanceof AxiosError) {
        io.logError(`${get(i18n).t('shell.wget.failed')}: ${err.response?.data}`)
        return 255
      } else {
        io.logError(`${get(i18n).t('shell.wget.failed')}: ${err}`)
        return 255
      }
    }
    io.logSuccess(get(i18n).t('shell.wget.success'))
    return 0
  }
}
