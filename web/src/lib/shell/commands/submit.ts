import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { RnixEnv } from '../shell'
import ansiColors from 'ansi-colors'
import { i18n } from '$lib/i18n'
import { get } from 'svelte/store'
import { AxiosError } from 'axios'
import { submitFlag } from '$lib/api/challenge'

export class Submit implements Command {
  name = 'submit'
  man = get(i18n).t('shell.submit.man')
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noGameSpecified'))}`)
      return 1
    } else if (envp.challenge == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.noChallengeSpecified'))}`)
      return 1
    }

    if (_args.length == 0) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.usage'))}`)
      return 1
    }
    const flag = origin.replace('submit ', '').trim()
    if (flag.length == 0) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.usage'))}`)
      return 1
    }
    io.println(`${ansiColors.green('[+]')} ` + ansiColors.dim(`${get(i18n).t('shell.submit.waiting')}: ${flag}`))

    try {
      let resp = await submitFlag({
        id: -1,
        created_at: 0,
        user_id: 0,
        challenge_id: envp.challenge?.id,
        content: flag,
        solved: false,
        with_score: false,
      })
      if (resp) {
        io.println(`${ansiColors.green('[+]')} ${ansiColors.dim(get(i18n).t('shell.submit.right'))}`)
        return 0
      } else {
        io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.wrong'))}`)
        return 1
      }
    } catch (err: unknown) {
      if (err instanceof AxiosError) {
        io.println(
          `${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.error'))}: ${err.response?.data}`
        )
        return 255
      } else {
        io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.error'))}: ${err}`)
        return 255
      }
    }
  }
}
