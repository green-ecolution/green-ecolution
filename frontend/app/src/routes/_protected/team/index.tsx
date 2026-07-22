import UserCard from '@/components/general/cards/UserCard'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import { createFileRoute } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { userQuery } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { pendingLoading } from '@/lib/router'

const TEAM_USERS_PARAMS = { page: 1, perPage: 100 }

export const Route = createFileRoute('/_protected/team/')({
  component: Team,
  pendingComponent: pendingLoading('Daten werden geladen'),
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(userQuery(TEAM_USERS_PARAMS)),
})

function Team() {
  const { data: userRes } = useSuspenseQuery(userQuery(TEAM_USERS_PARAMS))

  return (
    <div className="container mt-6">
      <ListPageHeader
        title="Alle Mitarbeitenden"
        description="Hier finden Sie eine Übersicht aller Mitarbeitenden und weitere Informationen zu deren Rollen und welche Führerscheinklasse sie besitzen. Diese Informationen sind wichtig, wenn Personen zu einem Einsatzplan eingeteilt werden sollen."
      />

      <section className="mt-10">
        <ListCardHeader columns="1fr 1.25fr 1fr 1fr">
          <p>Verfügbarkeit</p>
          <p>Name</p>
          <p>Organisation</p>
          <p>Führerscheinklasse</p>
        </ListCardHeader>
        <EntityList
          items={userRes.data}
          getKey={(user) => user.email}
          emptyMessage="Es wurden leider keine Mitarbeitenden gefunden."
          renderItem={(user) => <UserCard user={user} />}
        />
      </section>
    </div>
  )
}
