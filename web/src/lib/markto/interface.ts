export type MarkToType = 'html' | 'terminal'

export interface MarkToHtmlOptions {
  katex?: boolean
  prism?: boolean
  headingAnchors?: boolean
}

export type MarkToTerminalOptions = {
  image?: boolean
}
