import type { Meta, StoryObj } from '@storybook/react'
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
        label="Upload File"
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
        label="Tree Photo"
        description="PNG, JPG up to 10MB"
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
        label="Document"
        description="PDF files only"
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
        label="Upload Image"
        description="PNG, JPG up to 5MB"
        error="File size must be less than 5MB"
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
        label="Tree Photo"
        description="PNG, JPG up to 10MB"
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
        <h3 className="text-lg font-semibold">Import Tree Data</h3>
        <p className="text-sm text-muted-foreground">
          Upload a CSV file containing tree data to import into the system.
        </p>
        <FileUpload
          label="CSV File"
          description="Supported format: .csv"
          accept=".csv"
          required
          value={file}
          onChange={setFile}
        />
      </div>
    )
  },
}
