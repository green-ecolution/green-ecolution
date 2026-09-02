import { type VariantProps } from 'class-variance-authority'

import { Avatar, AvatarFallback, AvatarImage, avatarVariants } from './avatar'

export interface CommentAuthor {
  name: string
  avatarUrl?: string | null
}

function initialsFromName(name: string): string {
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')
}

interface CommentAuthorAvatarProps {
  author: CommentAuthor
  size?: VariantProps<typeof avatarVariants>['size']
}

export function CommentAuthorAvatar({ author, size = 'default' }: CommentAuthorAvatarProps) {
  return (
    <Avatar size={size}>
      {author.avatarUrl && <AvatarImage src={author.avatarUrl} alt={author.name} />}
      <AvatarFallback variant="user">{initialsFromName(author.name)}</AvatarFallback>
    </Avatar>
  )
}
