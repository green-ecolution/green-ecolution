import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { FileUpload } from '../src/components/ui/file-upload'

const meta: Meta<typeof FileUpload> = {
  title: 'UI/FileUpload',
  component: FileUpload,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <FileUpload
        label="Datei hochladen"
        value={file}
        onChange={setFile}
      />
    )
  },
}

export const WithDescription: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <FileUpload
        label="Baumfoto"
        description="PNG, JPG bis zu 10MB"
        accept="image/png,image/jpeg"
        value={file}
        onChange={setFile}
      />
    )
  },
}

export const Required: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <FileUpload
        label="Dokument"
        description="Nur PDF-Dateien"
        accept="application/pdf"
        required
        value={file}
        onChange={setFile}
      />
    )
  },
}

export const WithError: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <FileUpload
        label="Bild hochladen"
        description="PNG, JPG bis zu 5MB"
        error="Dateigröße darf 5MB nicht überschreiten"
        value={file}
        onChange={setFile}
      />
    )
  },
}

export const WithFile: Story = {
  render: function Render() {
    const mockFile = new File(['test content'], 'tree-photo.jpg', {
      type: 'image/jpeg',
    })
    Object.defineProperty(mockFile, 'size', { value: 2048000 })

    const [file, setFile] = useState<File | null>(mockFile)
    return (
      <FileUpload
        label="Baumfoto"
        description="PNG, JPG bis zu 10MB"
        value={file}
        onChange={setFile}
      />
    )
  },
}

export const ImportDataExample: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <div className="max-w-md space-y-4">
        <h3 className="text-lg font-semibold">Baumdaten importieren</h3>
        <p className="text-sm text-muted-foreground">
          Laden Sie eine CSV-Datei mit Baumdaten hoch, um diese in das System zu importieren.
        </p>
        <FileUpload
          label="CSV-Datei"
          description="Unterstütztes Format: .csv"
          accept=".csv"
          required
          value={file}
          onChange={setFile}
        />
      </div>
    )
  },
}

export const SensorDataImport: Story = {
  render: function Render() {
    const [file, setFile] = useState<File | null>(null)
    return (
      <div className="max-w-md space-y-4">
        <h3 className="text-lg font-semibold">Sensordaten importieren</h3>
        <p className="text-sm text-muted-foreground">
          Importieren Sie historische Sensordaten als JSON-Datei.
        </p>
        <FileUpload
          label="JSON-Datei"
          description="Unterstützte Formate: .json"
          accept=".json,application/json"
          required
          value={file}
          onChange={setFile}
        />
      </div>
    )
  },
}
