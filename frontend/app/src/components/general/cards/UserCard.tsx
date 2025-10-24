import React from 'react'
import { User } from '@green-ecolution/backend-client'
import Pill from '../Pill'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { getUserRoleDetails } from '@/hooks/details/useDetailsForUserRole'
import { getUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'

interface UserCard {
  user: User
}

const UserCard: React.FC<UserCard> = ({ user }) => {
  const statusDetails = getUserStatusDetails(user.status)

  return (
    <div className="bg-white border border-dark-50 p-6 rounded-xl shadow-cards flex flex-col gap-y-4 lg:py-4 lg:grid lg:grid-cols-[1fr,1.25fr,1fr,1fr] lg:items-center lg:gap-5 xl:px-10">
      <Pill
        label={statusDetails?.label ?? 'Keine Angabe'}
        theme={statusDetails?.color ?? 'dark-400'}
      />

      <h2 className="text-dark font-bold text-md mb-0.5">
        {user.firstName} {user.lastName}
      </h2>

      <p className="text-dark-800">
        <span className="lg:sr-only">Organisation:&nbsp;</span>
        {user.roles.map((role, index) => (
          <span key={getUserRoleDetails(role).label}>
            {getUserRoleDetails(role).label}
            {index < user.roles.length - 1 ? ', ' : ''}
          </span>
        ))}
      </p>

      <p className="text-dark-800">
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
      </p>
    </div>
  )
}

export default UserCard
