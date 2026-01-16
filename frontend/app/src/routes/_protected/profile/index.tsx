import { createFileRoute } from '@tanstack/react-router'
import { useUserStore } from '@/store/store'
import { UserRound } from 'lucide-react'
import { getUserRoleDetails } from '@/hooks/details/useDetailsForUserRole'
import { getUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { DrivingLicense, UserRole } from '@green-ecolution/backend-client'
import { Avatar, AvatarFallback, Badge, DetailedList } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/profile/')({
  component: Profile,
})

function Profile() {
  const user = useUserStore()

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Profil von {user.firstName} {user.lastName}
        </h1>
        <p>
          Dies ist Ihre persönliche Profilseite. Auf dieser Seite finden Sie Ihre persönlichen
          Daten.{' '}
        </p>
      </article>

      <section className="mt-16 grid grid-cols-1 gap-y-10 lg:grid-cols-2 lg:gap-x-11">
        <div className="flex items-center gap-x-6">
          <Avatar size="2xl" className="2xl:size-48">
            <AvatarFallback variant="guest" className="2xl:text-4xl">
              <UserRound className="w-12 h-12 2xl:w-16 2xl:h-16 text-white" />
            </AvatarFallback>
          </Avatar>
          <div>
            <h2 className="text-xl font-bold font-lato xl:text-2xl">
              {user.firstName} {user.lastName}
            </h2>
            <ul className="mt-2 flex flex-col gap-2 xl:mt-4">
              {user.userRoles?.length > 0 &&
                user.userRoles.map((role: UserRole) => (
                  <li key={getUserRoleDetails(role).label}>
                    <Badge variant="outline-green-light" size="lg">
                      {getUserRoleDetails(role).label}
                    </Badge>
                  </li>
                ))}
            </ul>
          </div>
        </div>
      </section>

      <DetailedList
        columns={2}
        className="mt-10 lg:mt-16"
        details={[
          { label: 'Username:', value: user.username ?? 'Keine Angabe' },
          {
            label: 'Verfügbarkeit:',
            value: getUserStatusDetails(user.userStatus).label,
          },
          { label: 'Vorname:', value: user.firstName ?? 'Keine Angabe' },
          {
            label: 'Führerscheinklasse:',
            value:
              user.drivingLicenses && user.drivingLicenses.length > 0
                ? user.drivingLicenses
                    .map((dl: DrivingLicense) => getDrivingLicenseDetails(dl).label)
                    .join(', ')
                : 'Keine Angabe',
          },
          { label: 'Nachname:', value: user.lastName ?? 'Keine Angabe' },
          {
            label: 'Rollen:',
            value: user.userRoles
              .map((role: UserRole) => getUserRoleDetails(role).label)
              .join(', '),
          },
          { label: 'E-Mail:', value: user.email ?? 'Keine Angabe' },
        ]}
      />
    </div>
  )
}
