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
      io.logError(get(i18n).t('shell.noGameSpecified'))
      io.logInfo(get(i18n).t('shell.noGameSpecifiedTips'))
      return 1
    } else if (envp.challenge == null) {
      io.logError(get(i18n).t('shell.noChallengeSpecified'))
      io.logInfo(get(i18n).t('shell.noChallengeSpecifiedTips'))
      return 1
    }
    const flag = origin.replace('submit ', '').trim()
    if (flag.length == 0) {
      io.logError(get(i18n).t('shell.submit.usage'))
      return 1
    }
    if (flag.length == 0) {
      io.logError(get(i18n).t('shell.submit.usage'))
      return 1
    }
    io.logInfo(`${get(i18n).t('shell.submit.waiting')}: ${flag}`)

    try {
      const resp = await submitFlag({
        id: -1,
        created_at: 0,
        user_id: 0,
        challenge_id: envp.challenge?.id,
        content: flag,
        solved: false,
        team_id: 0,
      })
      if (resp.result) {
        io.logSuccess(get(i18n).t('shell.submit.right'))
        if (resp.blood_state > 0 && resp.blood_state <= 3) {
          io.logSuccess(get(i18n).t('shell.submit.blood', { blood: resp.blood_state }))
        }
        return 0
      } else {
        io.logError(get(i18n).t('shell.submit.wrong'))
        return 1
      }
    } catch (err: unknown) {
      if (err instanceof AxiosError) {
        io.logError(`${ansiColors.dim(get(i18n).t('shell.submit.error'))}: ${err.response?.data}`)
        return 255
      } else {
        io.logError(`${ansiColors.dim(get(i18n).t('shell.submit.error'))}: ${err}`)
        return 255
      }
    }
  }
}
