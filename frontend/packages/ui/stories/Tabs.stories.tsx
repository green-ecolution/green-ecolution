import type { Meta, StoryObj } from '@storybook/react'
import { Trees, Truck, Radio, Droplets } from 'lucide-react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../src/components/ui/tabs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../src/components/ui/card'

const meta: Meta<typeof Tabs> = {
  title: 'UI/Tabs',
  component: Tabs,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Tabs defaultValue="account" className="w-[400px]">
      <TabsList>
        <TabsTrigger value="account">Account</TabsTrigger>
        <TabsTrigger value="password">Password</TabsTrigger>
      </TabsList>
      <TabsContent value="account">
        <Card>
          <CardHeader>
            <CardTitle>Account</CardTitle>
            <CardDescription>
              Make changes to your account here.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Account settings and preferences.
            </p>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="password">
        <Card>
          <CardHeader>
            <CardTitle>Password</CardTitle>
            <CardDescription>
              Change your password here.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Password and security settings.
            </p>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  ),
}

export const WithIcons: Story = {
  render: () => (
    <Tabs defaultValue="trees" className="w-[500px]">
      <TabsList className="grid w-full grid-cols-4">
        <TabsTrigger value="trees" className="flex items-center gap-2">
          <Trees className="h-4 w-4" />
          Trees
        </TabsTrigger>
        <TabsTrigger value="vehicles" className="flex items-center gap-2">
          <Truck className="h-4 w-4" />
          Vehicles
        </TabsTrigger>
        <TabsTrigger value="sensors" className="flex items-center gap-2">
          <Radio className="h-4 w-4" />
          Sensors
        </TabsTrigger>
        <TabsTrigger value="watering" className="flex items-center gap-2">
          <Droplets className="h-4 w-4" />
          Watering
        </TabsTrigger>
      </TabsList>
      <TabsContent value="trees">
        <Card>
          <CardHeader>
            <CardTitle>Trees</CardTitle>
            <CardDescription>Manage your tree inventory.</CardDescription>
          </CardHeader>
          <CardContent>
            <p>1,234 trees registered in the system.</p>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="vehicles">
        <Card>
          <CardHeader>
            <CardTitle>Vehicles</CardTitle>
            <CardDescription>Fleet management.</CardDescription>
          </CardHeader>
          <CardContent>
            <p>8 vehicles in operation.</p>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="sensors">
        <Card>
          <CardHeader>
            <CardTitle>Sensors</CardTitle>
            <CardDescription>IoT sensor network.</CardDescription>
          </CardHeader>
          <CardContent>
            <p>56 sensors connected.</p>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="watering">
        <Card>
          <CardHeader>
            <CardTitle>Watering Plans</CardTitle>
            <CardDescription>Irrigation schedules.</CardDescription>
          </CardHeader>
          <CardContent>
            <p>3 active watering plans.</p>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  ),
}

export const FullWidth: Story = {
  render: () => (
    <Tabs defaultValue="overview" className="w-full">
      <TabsList className="w-full">
        <TabsTrigger value="overview" className="flex-1">Overview</TabsTrigger>
        <TabsTrigger value="analytics" className="flex-1">Analytics</TabsTrigger>
        <TabsTrigger value="reports" className="flex-1">Reports</TabsTrigger>
      </TabsList>
      <TabsContent value="overview">
        <p className="pt-4">Overview content here.</p>
      </TabsContent>
      <TabsContent value="analytics">
        <p className="pt-4">Analytics content here.</p>
      </TabsContent>
      <TabsContent value="reports">
        <p className="pt-4">Reports content here.</p>
      </TabsContent>
    </Tabs>
  ),
}
