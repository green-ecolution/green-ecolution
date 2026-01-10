import type { Meta, StoryObj } from '@storybook/react'
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from 'lucide-react'
import { Alert, AlertDescription, AlertTitle } from '../src/components/ui/alert'

const meta: Meta<typeof Alert> = {
  title: 'UI/Alert',
  component: Alert,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'destructive', 'warning', 'success'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Alert>
      <Info />
      <AlertTitle>Information</AlertTitle>
      <AlertDescription>
        This is an informational alert message.
      </AlertDescription>
    </Alert>
  ),
}

export const Destructive: Story = {
  render: () => (
    <Alert variant="destructive">
      <AlertCircle />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>
        Something went wrong. Please try again later.
      </AlertDescription>
    </Alert>
  ),
}

export const Warning: Story = {
  render: () => (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Warning</AlertTitle>
      <AlertDescription>
        Please review your input before proceeding.
      </AlertDescription>
    </Alert>
  ),
}

export const Success: Story = {
  render: () => (
    <Alert variant="success">
      <CheckCircle2 />
      <AlertTitle>Success</AlertTitle>
      <AlertDescription>
        Your changes have been saved successfully.
      </AlertDescription>
    </Alert>
  ),
}

export const AllVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <Alert>
        <Info />
        <AlertTitle>Default</AlertTitle>
        <AlertDescription>Default alert variant.</AlertDescription>
      </Alert>
      <Alert variant="success">
        <CheckCircle2 />
        <AlertTitle>Success</AlertTitle>
        <AlertDescription>Success alert variant.</AlertDescription>
      </Alert>
      <Alert variant="warning">
        <TriangleAlert />
        <AlertTitle>Warning</AlertTitle>
        <AlertDescription>Warning alert variant.</AlertDescription>
      </Alert>
      <Alert variant="destructive">
        <AlertCircle />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>Destructive alert variant.</AlertDescription>
      </Alert>
    </div>
  ),
}

export const SensorNotice: Story = {
  render: () => (
    <Alert>
      <Info />
      <AlertTitle>Hinweis: Dieser Baum ist nicht mit einem Sensor ausgestattet.</AlertTitle>
      <AlertDescription>
        Dieser Baum wurde bisher nicht mit einem Sensor ausgestattet, sodass keine
        Informationen über den aktuellen Bewässerungszustand angezeigt werden können.
        Aus diesem Grund wird der Bewässerungszustand als unbekannt ausgezeichnet.
      </AlertDescription>
    </Alert>
  ),
}
