import { Link, useMatches } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { CircleQuestionMark } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import { chapterForRoute } from '@/lib/handbook/contextHelp'

function ContextHelpLink() {
  const { t } = useTranslation('help')
  const matches = useMatches()
  const slug = chapterForRoute(matches[matches.length - 1]?.fullPath)

  return (
    <Button asChild variant="ghost" size="icon" aria-label={t('chapter.contextHelp')}>
      {slug ? (
        <Link to="/help/$slug" params={{ slug }}>
          <CircleQuestionMark className="size-5" />
        </Link>
      ) : (
        <Link to="/help">
          <CircleQuestionMark className="size-5" />
        </Link>
      )}
    </Button>
  )
}

export default ContextHelpLink
