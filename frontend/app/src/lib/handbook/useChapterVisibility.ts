import { useQuery } from '@tanstack/react-query'
import { modulesQuery } from '@/api/queries'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { activeModuleKeys } from '@/modules/activeModules'
import { handbookIndex } from './index'
import { chapterFilter, type ChapterFilter } from './visibility'

/** /help is public, so this never suspends and asks for modules only when logged in. */
export function useChapterVisibility(): ChapterFilter {
  const { isAuthenticated } = useAuthSession()
  const { data } = useQuery({ ...modulesQuery(), enabled: isAuthenticated })

  return chapterFilter(handbookIndex.chapters, {
    loggedIn: isAuthenticated,
    activeKeys: data ? activeModuleKeys(data) : undefined,
  })
}
