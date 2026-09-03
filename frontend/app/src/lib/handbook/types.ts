export type LinkTarget =
  | { kind: 'external'; href: string }
  | { kind: 'chapter'; slug: string; anchor?: string }
  | { kind: 'app'; to: string }

export type Inline =
  | { kind: 'text'; value: string }
  | { kind: 'code'; value: string }
  | { kind: 'strong'; children: Inline[] }
  | { kind: 'emphasis'; children: Inline[] }
  | { kind: 'link'; target: LinkTarget; children: Inline[] }

export type Block =
  | { kind: 'heading'; level: 2 | 3; text: string; anchor: string }
  | { kind: 'paragraph'; children: Inline[] }
  | { kind: 'list'; items: Inline[][] }
  | { kind: 'steps'; items: Inline[][] }
  | { kind: 'callout'; tone: 'note' | 'tip' | 'important' | 'warning'; children: Block[] }
  | { kind: 'figure'; image: string; caption: string }
  | { kind: 'table'; head: Inline[][]; rows: Inline[][][] }
  | { kind: 'code'; language: string | null; value: string }

export interface ChapterSection {
  anchor: string
  title: string
  level: 2 | 3
}

export interface ChapterMeta {
  slug: string
  title: string
  part: string
  summary: string
  routes: string[]
  sections: ChapterSection[]
}

export interface HandbookPart {
  id: string
  title: string
  chapters: string[]
}

export interface HandbookIndex {
  parts: HandbookPart[]
  chapters: Record<string, ChapterMeta>
}

export interface ChapterContent {
  slug: string
  blocks: Block[]
}

export interface SearchEntry {
  slug: string
  anchor: string
  sectionTitle: string
  text: string
}
