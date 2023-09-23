import { parse, type ParseEntry } from 'shell-quote'
import stringWidth from './unicode'
import color from 'ansi-colors'

export function countLines(input: string, maxCols: number): number {
  return offsetToColRow(input, stringWidth(input), maxCols).row + 1
}

export function offsetToColRow(input: string, offset: number, maxCols: number): { col: number; row: number } {
  let col = 0
  let row = 0

  // console.log(input, offset, maxCols)
  const stripedInput = color.unstyle(input)
  let currentOffset = 0
  for (let i = 0; i < stripedInput.length && currentOffset < offset; i++) {
    const chr = stripedInput.charAt(i)
    if (chr === '\n') {
      col = 0
      row += 1
    } else {
      col += stringWidth(chr)
      if (col >= maxCols) {
        col = 0
        row += 1
      }
      // col += 1
    }
    currentOffset += stringWidth(chr)
    // console.log(i, chr, col, row)
  }

  // console.log('offsetToColRow', col, row)

  return { col, row }
}

/**
 * Checks if there is an incomplete input
 *
 * An incomplete input is considered:
 * - An input that contains unterminated single quotes
 * - An input that contains unterminated double quotes
 * - An input that ends with "\"
 * - An input that has an incomplete boolean shell expression (&& and ||)
 * - An incomplete pipe expression (|)
 */
export function isIncompleteInput(input: string) {
  // Empty input is not incomplete
  if (input.trim() == '') {
    return false
  }

  // Check for dangling single-quote strings
  if ((input.match(/'/g) || []).length % 2 !== 0) {
    return true
  }
  // Check for dangling double-quote strings
  if ((input.match(/"/g) || []).length % 2 !== 0) {
    return true
  }
  // Check for dangling boolean or pipe operations
  if (
    input
      .split(/(\|\||\||&&)/g)
      .pop()
      ?.trim() == ''
  ) {
    return true
  }
  // Check for tailing slash
  return input.endsWith('\\') && !input.endsWith('\\\\')
}

export function hasTailingWhitespace(input: string): boolean {
  return input.match(/[^\\][ \t]$/m) !== null
}

export function getLastToken(input: string): ParseEntry {
  if (input.trim() === '') return ''
  if (hasTailingWhitespace(input)) return ''

  const tokens: ParseEntry[] = parse(input)
  return tokens.pop() || ''
}

export function getSharedFragment(fragment: string, candidates: string[]): string {
  if (fragment.length >= candidates[0].length) return fragment

  const oldFragment = fragment

  fragment += candidates[0].slice(fragment.length, fragment.length + 1)

  for (let i = 0; i < candidates.length; i++) {
    // this is wrong candidate
    if (!candidates[i].startsWith(oldFragment)) return ''

    if (!candidates[i].startsWith(fragment)) return oldFragment
  }

  return getSharedFragment(fragment, candidates)
}
