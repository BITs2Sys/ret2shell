export interface Wiki {
  id: number
  title: string
  published_at: number
  updated_at: number
  author_id: number
  content: string
  parent: number
}

export interface WikiEntry {
  id: number
  title: string
  parent: number | null
  addr: string
  children: WikiEntry[]
}

export function transformToWikiEntry(wikis: Wiki[]): WikiEntry[] {
  return wikis.map((item) => {
    return {
      id: item.id,
      title: item.title,
      addr: `/wiki/${item.id}`,
      parent: item.parent,
      children: [],
    }
  })
}
