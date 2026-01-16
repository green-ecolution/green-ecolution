import React from 'react'
import { User } from '@green-ecolution/backend-client'
import {
  Badge,
  ListCard,
  ListCardTitle,
  ListCardDescription,
} from '@green-ecolution/ui'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { getUserRoleDetails } from '@/hooks/details/useDetailsForUserRole'
import { getUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'

interface UserCard {
  user: User
}

const UserCard: React.FC<UserCard> = ({ user }) => {
  const statusDetails = getUserStatusDetails(user.status)

  return (
    <ListCard columns="1fr 1.25fr 1fr 1fr" hoverable={false} className="lg:py-4">
      <div>
        <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
          {statusDetails?.label ?? 'Keine Angabe'}
        </Badge>
      </div>

      <ListCardTitle className="text-md">
        {user.firstName} {user.lastName}
      </ListCardTitle>

      <ListCardDescription>
        <span className="lg:sr-only">Organisation:&nbsp;</span>
        {user.roles.map((role, index) => (
          <span key={getUserRoleDetails(role).label}>
            {getUserRoleDetails(role).label}
            {index < user.roles.length - 1 ? ', ' : ''}
          </span>
        ))}
      </ListCardDescription>

      <ListCardDescription>
        <span className="lg:sr-only">Führerscheinklasse:&nbsp;</span>
        {user.drivingLicenses && user.drivingLicenses.length > 0 ? (
          <>
            {user.drivingLicenses.map((drivingLicense, index) => (
              <span key={getDrivingLicenseDetails(drivingLicense).label}>
                {getDrivingLicenseDetails(drivingLicense).label}
                {index < user.drivingLicenses.length - 1 ? ', ' : ''}
              </span>
            ))}
          </>
        ) : (
          'Keine Angabe'
        )}
      </ListCardDescription>
    </ListCard>
  )
}

export default UserCard
