import stripAnsi from 'strip-ansi'
import { eastAsianWidth } from './unicodeWidth'
import emojiRegex from 'emoji-regex'

export default function stringWidth(str: string, ignoreReturn?: boolean, ambiguousIsNarrow?: boolean) {
  if (str.length === 0) {
    return 0
  }

  str = stripAnsi(str)

  if (str.length === 0) {
    return 0
  }

  if (!ambiguousIsNarrow) {
    ambiguousIsNarrow = true
  }

  str = str.replace(emojiRegex(), '  ')
  const ambiguousCharacterWidth = ambiguousIsNarrow ? 1 : 2
  let width = 0

  for (const character of str) {
    const codePoint = character.codePointAt(0) as number

    // Ignore control characters, if code is '\n', do not ignore it so that we can count it as a new line.
    if ((codePoint <= 0x1f && codePoint != 0x0a) || (codePoint >= 0x7f && codePoint <= 0x9f)) continue

    if (ignoreReturn && codePoint == 0x0a) continue

    // Ignore combining characters
    if (codePoint >= 0x300 && codePoint <= 0x36f) continue

    const code = eastAsianWidth(character)
    switch (code) {
      case 'F':
      case 'W':
        width += 2
        break
      case 'A':
        width += ambiguousCharacterWidth
        break
      default:
        width += 1
    }
  }

  return width
}
