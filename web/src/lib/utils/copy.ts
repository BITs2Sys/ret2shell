export async function copyToClipboard(text: string) {
  if (window.navigator.clipboard) {
    return window.navigator.clipboard.writeText(text)
  } else {
    const copy = await import('copy-to-clipboard')
    if (!copy.default(text)) {
      throw Error('Copy failed')
    }
    return Promise.resolve()
  }
}
