import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { UserRound } from 'lucide-react'
import { userQueries } from '@/api/queries'
import { useUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'
import { useDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { DrivingLicense } from '@green-ecolution/backend-client'
import { useCurrentUserAvatar } from '@/lib/auth/useCurrentUserAvatar'
import { Avatar, AvatarFallback, AvatarImage, Badge, DetailedList } from '@green-ecolution/ui'
import LanguageSwitcher from '@/components/settings/LanguageSwitcher'
import { roleDisplayName } from '@/components/settings/roles/roleList'

export const Route = createFileRoute('/_protected/settings/profile/')({
  component: Profile,
})

function Profile() {
  const { t } = useTranslation(['settings', 'common'])
  const user = useCurrentUser()
  const avatarUrl = useCurrentUserAvatar()
  const { data: me } = useQuery(userQueries.me())
  const roles = me?.roles ?? []
  const getUserStatusDetails = useUserStatusDetails()
  const getDrivingLicenseDetails = useDrivingLicenseDetails()

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('profile.heading', { firstName: user.firstName, lastName: user.lastName })}
        </h1>
        <p>{t('profile.intro')}</p>
      </article>

      <section className="mt-16 grid grid-cols-1 gap-y-10 lg:grid-cols-2 lg:gap-x-11">
        <div className="flex items-center gap-x-6">
          <Avatar size="2xl" className="2xl:size-48">
            {avatarUrl && (
              <AvatarImage src={avatarUrl} alt={`${user.firstName} ${user.lastName}`} />
            )}
            <AvatarFallback variant="guest" className="2xl:text-4xl">
              <UserRound className="w-12 h-12 2xl:w-16 2xl:h-16 text-white" />
            </AvatarFallback>
          </Avatar>
          <div>
            <h2 className="text-xl font-bold font-lato xl:text-2xl">
              {user.firstName} {user.lastName}
            </h2>
            {me?.organization && (
              <div className="mt-2 xl:mt-4">
                <Badge variant="outline-green-light" size="lg">
                  {me.organization.name}
                </Badge>
              </div>
            )}
          </div>
        </div>
      </section>

      <DetailedList
        columns={2}
        className="mt-10 lg:mt-16"
        details={[
          { label: t('profile.usernameLabel'), value: user.username ?? t('common:state.noData') },
          {
            label: t('profile.availabilityLabel'),
            value: getUserStatusDetails(user.userStatus).label,
          },
          { label: t('profile.firstNameLabel'), value: user.firstName ?? t('common:state.noData') },
          {
            label: t('profile.drivingLicenseLabel'),
            value:
              user.drivingLicenses && user.drivingLicenses.length > 0
                ? user.drivingLicenses
                    .map((dl: DrivingLicense) => getDrivingLicenseDetails(dl).label)
                    .join(', ')
                : t('common:state.noData'),
          },
          { label: t('profile.lastNameLabel'), value: user.lastName ?? t('common:state.noData') },
          {
            label: t('profile.rolesLabel'),
            value:
              roles.length > 0
                ? roles.map((role) => roleDisplayName(role, t)).join(', ')
                : t('common:state.noData'),
          },
          { label: t('profile.emailLabel'), value: user.email ?? t('common:state.noData') },
        ]}
      />

      <section className="mt-16">
        <LanguageSwitcher />
      </section>
    </div>
  )
}
