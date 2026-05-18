import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ChevronDown, Trash2 } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogIcon,
  AlertDialogTitle,
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  toast,
} from '@green-ecolution/ui'
import { sensorApi } from '@/api/backendApi'

interface SensorActionsMenuProps {
  sensorId: string
}

const SensorActionsMenu = ({ sensorId }: SensorActionsMenuProps) => {
  const [dialogOpen, setDialogOpen] = useState(false)
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const deleteMutation = useMutation({
    mutationFn: () => sensorApi.deleteSensor({ sensorId }),
    onSuccess: async () => {
      toast.success('Sensor wurde gelöscht.')
      await queryClient.invalidateQueries({ queryKey: ['sensors'] })
      await navigate({ to: '/sensors', search: { page: 1 } })
    },
    onError: () => {
      toast.error('Der Sensor konnte nicht gelöscht werden.')
    },
  })

  return (
    <AlertDialog open={dialogOpen} onOpenChange={setDialogOpen}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className="gap-2 [&_svg]:size-4">
            Aktionen
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-56">
          <DropdownMenuItem
            className="text-destructive focus:text-destructive cursor-pointer"
            onSelect={(e) => {
              e.preventDefault()
              setDialogOpen(true)
            }}
          >
            <Trash2 className="mr-2 size-4" />
            Sensor löschen
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogIcon variant="destructive">
            <Trash2 />
          </AlertDialogIcon>
          <AlertDialogTitle>Sensor wirklich löschen?</AlertDialogTitle>
          <AlertDialogDescription>
            Diese Aktion kann nicht rückgängig gemacht werden. Der Sensor{' '}
            <code className="font-mono text-foreground">{sensorId}</code> wird inklusive seiner
            LoRaWAN-Konfiguration und aller Messdaten dauerhaft entfernt.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={deleteMutation.isPending}>Abbrechen</AlertDialogCancel>
          <AlertDialogAction
            disabled={deleteMutation.isPending}
            onClick={(e) => {
              e.preventDefault()
              deleteMutation.mutate()
            }}
          >
            {deleteMutation.isPending ? 'Wird gelöscht …' : 'Endgültig löschen'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

export default SensorActionsMenu
