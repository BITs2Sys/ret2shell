// This module provides shell entrypoint.
import { RnixStdio } from './stdio'
import { Exec } from './exec'
import type { Terminal } from 'xterm'
import ansiColors from 'ansi-colors'

export class RnixShell {
  private readonly proxy: RnixStdio
  private exec: Exec
  private code = 0

  public constructor(term: Terminal) {
    ansiColors.enabled = true
    this.proxy = new RnixStdio(term)
    this.exec = new Exec()
  }
}
