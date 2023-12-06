import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { Challenge, Tag } from '$lib/models/challenge'
import { getTagList } from '$lib/api/v1/challenge'
import type { RnixEnv } from '../shell'
import ansiColors from 'ansi-colors'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import { unicodeStrDisplayLength } from '../utils/unicodeWidth'
import ansiEscapes from 'isomorphic-ansi-escapes'

export class Ls implements Command {
  name = 'ls'
  man = get(i18n).t('shell.ls.man')
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.logError(get(i18n).t('shell.noGameSpecified'))
      io.logInfo(get(i18n).t('shell.noGameSpecifiedTips'))
      return 1
    }
    const map = new Map<number, Array<Challenge>>()
    envp.availableChallenges.forEach((challenge) => {
      if (!map.has(challenge.tag_id)) {
        map.set(challenge.tag_id, [])
      }
      map.get(challenge.tag_id)?.push(challenge)
    })
    let tags: Tag[] = []
    try {
      tags = (await getTagList()).toSorted((a, b) => (a.name > b.name ? 1 : a.name === b.name ? 0 : -1))
    } catch {
      tags = []
    }
    Array.from(map.entries()).map(([tagId, challenges]) => {
      io.println(ansiColors.bold.blue(tags.find((tag) => tag.id === tagId)?.name || '[UNGROUPED]'))
      const maxNameLength = challenges.reduce(
        (max, challenge) => Math.max(max, unicodeStrDisplayLength(challenge.name)),
        0
      )
      const itemsPerLine = Math.floor(io.termWidth() / (maxNameLength + 2))
      let iter = 0
      for (const challenge of challenges) {
        // io.print(`${ansiEscapes.link(challenge.name, `rnix-chal://${challenge.id}`)}`)
        io.print(
          `${ansiEscapes.link(challenge.name, `rnix-chal://${challenge.id}`)}${' '.repeat(
            maxNameLength - unicodeStrDisplayLength(challenge.name)
          )}  `
        )
        iter++
        if (iter % itemsPerLine == 0) {
          io.println('')
        }
      }
      io.println('')
      if (iter % itemsPerLine != 0) {
        io.println('')
      }
    })
    return 0
  }
}
