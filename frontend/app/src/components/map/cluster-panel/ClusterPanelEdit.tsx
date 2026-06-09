import { Controller } from 'react-hook-form'
import { X } from 'lucide-react'
import {
  Button,
  FormField,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  TextareaField,
} from '@green-ecolution/ui'
import { SoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'
import type { TreeClusterResponse } from '@/api/backendApi'
import { useClusterPanelEdit } from './useClusterPanelEdit'

interface ClusterPanelEditProps {
  treecluster: TreeClusterResponse
  onCancel: () => void
  onClose: () => void
  onSaved: () => void
}

const ClusterPanelEdit = ({ treecluster, onCancel, onClose, onSaved }: ClusterPanelEditProps) => {
  const { form, onSubmit, isPending } = useClusterPanelEdit(treecluster, { onSaved })
  const {
    register,
    control,
    formState: { errors },
  } = form

  return (
    <form
      className="flex flex-col gap-y-5"
      onSubmit={(event) => {
        event.preventDefault()
        void onSubmit()
      }}
    >
      <header className="flex items-center justify-between gap-3">
        <h2 className="font-lato text-xl font-bold text-dark-900">Gruppe bearbeiten</h2>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Seitenansicht schließen"
          className="hidden rounded-full bg-dark-50 text-dark-500 hover:bg-dark-100 hover:text-dark-700 lg:flex"
          onClick={onClose}
        >
          <X />
        </Button>
      </header>

      <FormField label="Name" required error={errors.name?.message} {...register('name')} />
      <FormField
        label="Adresse"
        required
        error={errors.address?.message}
        {...register('address')}
      />

      <Controller
        name="soilCondition"
        control={control}
        render={({ field }) => (
          <div className="flex flex-col gap-y-2">
            <Label htmlFor="soilCondition">
              Bodenbeschaffenheit
              <span className="text-destructive ml-1">*</span>
            </Label>
            <Select value={field.value} onValueChange={field.onChange}>
              <SelectTrigger id="soilCondition">
                <SelectValue placeholder="Wählen Sie eine Bodenbeschaffenheit aus" />
              </SelectTrigger>
              <SelectContent>
                {SoilConditionOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.soilCondition?.message && (
              <p className="text-sm text-destructive">{errors.soilCondition.message}</p>
            )}
          </div>
        )}
      />

      <TextareaField
        label="Kurze Beschreibung"
        placeholder="Hier ist Platz für Notizen"
        error={errors.description?.message}
        {...register('description')}
      />

      <div className="flex gap-3 mt-2">
        <Button type="submit" disabled={isPending}>
          Speichern
        </Button>
        <Button type="button" variant="outline" onClick={onCancel}>
          Abbrechen
        </Button>
      </div>
    </form>
  )
}

export default ClusterPanelEdit
