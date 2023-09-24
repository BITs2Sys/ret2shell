import type { ParseEntry } from 'shell-quote'
import type { RnixStdio } from '../stdio'
import type { Command } from './interface'
import type { Challenge } from '$lib/models/challenge'
import { getChallengeList, getTagList } from '$lib/api/challenge'
import type { RnixEnv } from '../shell'
import ansiColors from 'ansi-colors'
import { get } from 'svelte/store'
import { i18n } from '$lib/i18n'
import ansiEscapes from 'isomorphic-ansi-escapes'

export class Ls implements Command {
  name = 'ls'
  man = 'ls'
  func = async (io: RnixStdio, _args: ParseEntry[], origin: string, envp: RnixEnv) => {
    if (envp.game == null) {
      io.println(`${ansiColors.red('[-]')} ${ansiColors.dim(get(i18n).t('shell.submit.no_game'))}`)
      return 1
    }
    const map = new Map<number, Array<Challenge>>()
    const challenges = (await getChallengeList(envp.game?.id)).challenges
    challenges.forEach((challenge) => {
      if (!map.has(challenge.tag_id)) {
        map.set(challenge.tag_id, [])
      }
      map.get(challenge.tag_id)?.push(challenge)
    })
    let tags = await getTagList()
    Array.from(map.entries()).map(([tagId, challenges]) => {
      io.println(ansiColors.bold.blue(tags.find((tag) => tag.id === tagId)?.name || 'UNKNOWN TAG'))
      return challenges.map((challenge) => {
        io.println(`    ${ansiEscapes.link(challenge.name, `challenge://${challenge.id}`)}`)
      })
    })
    return 0
  }
}
