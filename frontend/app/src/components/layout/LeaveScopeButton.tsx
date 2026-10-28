import { useTranslation } from 'react-i18next'
import { Button } from '@green-ecolution/ui'
import { useOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { useSwitchOrganizationScope } from '@/lib/auth/scope/useSwitchOrganizationScope'

function LeaveScopeAction() {
  const { t } = useTranslation('navigation')
  const switchScope = useSwitchOrganizationScope()

  return (
    <Button variant="ghost" onClick={() => void switchScope(null)}>
      {t('user.leaveScope')}
    </Button>
  )
}

/** A way out for pages that only fail because the selected scope hides what they show. */
function LeaveScopeButton() {
  const scope = useOrganizationScope()
  return scope ? <LeaveScopeAction /> : null
}

export default LeaveScopeButton
