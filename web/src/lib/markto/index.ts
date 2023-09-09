import { type Processor, unified } from 'unified'
import type { MarkToHtmlOptions, MarkToTerminalOptions } from './interface'
import remarkParse from 'remark-parse'
import rehypeSlug from 'rehype-slug'
import rehypeAutolinkHeadings from 'rehype-autolink-headings'

type IParams = { type: 'html'; options?: MarkToHtmlOptions } | { type: 'terminal'; options?: MarkToTerminalOptions }

export class MarkTo {
  private processor: Processor | undefined

  public constructor() {}

  public async init(params: IParams) {
    this.processor = unified().use(remarkParse)
    switch (params.type) {
      case 'html':
        await this.initHtml(params.options as MarkToHtmlOptions)
        break
      case 'terminal':
        await this.initTerminal(params.options as MarkToTerminalOptions)
        break
    }
  }

  public async render(markdown: string) {
    // console.log(this.processor?.attachers)
    const result = await this.processor?.process(markdown)
    return result?.toString()
  }

  private async initHtml(options?: MarkToHtmlOptions) {
    const [remarkRehype, rehypeStringify] = await Promise.all([import('remark-rehype'), import('rehype-stringify')])
    /* remark */ {
      if (options?.katex) {
        const remarkMath = await import('remark-math')
        this.processor?.use(remarkMath.default)
      }
    }

    this.processor?.use(remarkRehype.default)

    /* rehype */ {
      if (options?.katex) {
        const rehypeKatex = await import('rehype-katex')
        this.processor?.use(rehypeKatex.default)
      }
      if (options?.prism) {
        const rehypePrismPlus = await import('rehype-prism-plus/common')
        this.processor?.use(rehypePrismPlus.default, { ignoreMissing: true })
      }
      this.processor?.use(rehypeSlug)
      this.processor?.use(rehypeAutolinkHeadings, { behavior: 'wrap' })
      this.processor?.use(rehypeStringify.default)
    }
  }

  private async initTerminal(_options?: MarkToTerminalOptions) {}
}
