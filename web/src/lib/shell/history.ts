export class BufferHistory {
  private readonly entries: string[]
  private cursor: number
  private buffer: string

  constructor() {
    this.cursor = 0
    this.buffer = ''
    this.entries = []
  }

  push(entry: string) {
    // check entry empty
    if (entry.trim() === '') return
    // check last entry is same as current
    const lastEntry = this.entries[this.entries.length - 1]
    if (entry === lastEntry) return

    this.entries.push(entry)
    this.cursor = this.entries.length
  }

  previous(buffer?: string) {
    // save user's current input
    if (this.cursor === this.entries.length && buffer) {
      // console.log('saving buffer')
      this.buffer = buffer
    }
    if (this.entries.length === 0) return this.buffer
    this.cursor = Math.max(0, this.cursor - 1)
    return this.entries[this.cursor]
  }

  next() {
    // recover user's buffer
    if (this.cursor + 1 >= this.entries.length) {
      this.cursor = this.entries.length
      return this.buffer
    }
    if (this.entries.length === 0) return this.buffer
    this.cursor = Math.min(this.entries.length, this.cursor + 1)
    return this.entries[this.cursor]
  }

  rewind() {
    this.cursor = this.entries.length
  }
}
