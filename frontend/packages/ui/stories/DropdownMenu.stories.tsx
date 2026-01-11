import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  LogOut,
  LogIn,
  Mail,
  MessageSquare,
  Plus,
  Settings,
  User,
  UserPlus,
  UserRound,
  Trees,
  Truck,
  Edit,
  Trash2,
  MoreHorizontal,
  ChevronDown,
} from 'lucide-react'
import { Avatar, AvatarFallback } from '../src/components/ui/avatar'
import { Button } from '../src/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from '../src/components/ui/dropdown-menu'

const meta: Meta<typeof DropdownMenu> = {
  title: 'UI/DropdownMenu',
  component: DropdownMenu,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">Menü öffnen</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56">
        <DropdownMenuLabel>Mein Konto</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem>
            <User className="mr-2 h-4 w-4" />
            <span>Profil</span>
            <DropdownMenuShortcut>⇧⌘P</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem>
            <Settings className="mr-2 h-4 w-4" />
            <span>Einstellungen</span>
            <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuItem>
          <LogOut className="mr-2 h-4 w-4" />
          <span>Abmelden</span>
          <DropdownMenuShortcut>⇧⌘Q</DropdownMenuShortcut>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  ),
}

export const WithSubMenu: Story = {
  render: () => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">Menü öffnen</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56">
        <DropdownMenuLabel>Aktionen</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem>
            <Plus className="mr-2 h-4 w-4" />
            <span>Neues Element</span>
          </DropdownMenuItem>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>
              <UserPlus className="mr-2 h-4 w-4" />
              <span>Benutzer einladen</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuPortal>
              <DropdownMenuSubContent>
                <DropdownMenuItem>
                  <Mail className="mr-2 h-4 w-4" />
                  <span>Per E-Mail</span>
                </DropdownMenuItem>
                <DropdownMenuItem>
                  <MessageSquare className="mr-2 h-4 w-4" />
                  <span>Per Nachricht</span>
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuPortal>
          </DropdownMenuSub>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  ),
}

export const TreeActionsMenu: Story = {
  render: () => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon">
          <MoreHorizontal className="h-4 w-4" />
          <span className="sr-only">Menü öffnen</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Baum-Aktionen</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem>
          <Edit className="mr-2 h-4 w-4" />
          Baum bearbeiten
        </DropdownMenuItem>
        <DropdownMenuItem>
          <Trees className="mr-2 h-4 w-4" />
          Details anzeigen
        </DropdownMenuItem>
        <DropdownMenuItem>
          <Truck className="mr-2 h-4 w-4" />
          Zum Bewässerungsplan hinzufügen
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="text-destructive focus:text-destructive">
          <Trash2 className="mr-2 h-4 w-4" />
          Baum löschen
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  ),
}

export const WithCheckboxItems: Story = {
  render: () => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">Status filtern</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56">
        <DropdownMenuLabel>Baumstatus</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem checked>Gesund</DropdownMenuCheckboxItem>
        <DropdownMenuCheckboxItem checked>Bewässerung nötig</DropdownMenuCheckboxItem>
        <DropdownMenuCheckboxItem>Kritisch</DropdownMenuCheckboxItem>
        <DropdownMenuCheckboxItem>Unbekannt</DropdownMenuCheckboxItem>
      </DropdownMenuContent>
    </DropdownMenu>
  ),
}

export const WithRadioItems: Story = {
  render: () => (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">Sortieren nach</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56">
        <DropdownMenuLabel>Bäume sortieren</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuRadioGroup value="name">
          <DropdownMenuRadioItem value="name">Name</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="status">Status</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="date">Pflanzjahr</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="location">Standort</DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  ),
}

export const TableRowActions: Story = {
  render: () => (
    <div className="flex items-center justify-between rounded-lg border p-4">
      <div>
        <p className="font-medium">Eiche #1234</p>
        <p className="text-sm text-muted-foreground">Hauptstraße 42</p>
      </div>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="sm">
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem>Details anzeigen</DropdownMenuItem>
          <DropdownMenuItem>Bearbeiten</DropdownMenuItem>
          <DropdownMenuItem>Sensor zuweisen</DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem className="text-destructive">Löschen</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  ),
}

export const UserProfileDropdown: Story = {
  render: () => (
    <div className="flex justify-end">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button type="button" className="group flex items-center gap-x-1">
            <Avatar>
              <AvatarFallback variant="user">CH</AvatarFallback>
            </Avatar>
            <ChevronDown className="w-5 h-5 text-dark transition-transform duration-300 group-data-[state=open]:rotate-180" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-72 bg-dark border-none shadow-cards text-white text-sm pt-5 px-2"
        >
          <div className="border-b border-dark-800 mx-3 pb-4 mb-2">
            <p>Angemeldet als:</p>
            <p className="font-semibold truncate">choffmann@green-ecolution.de</p>
          </div>
          <DropdownMenuItem className="text-light text-base p-3.5 rounded-2xl transition-all duration-300 hover:bg-green-light/20 hover:text-green-light-200 focus:bg-green-light/20 focus:text-green-light-200 cursor-pointer">
            <Settings className="mr-2 h-5 w-5" />
            <span className="font-lato font-semibold tracking-[0.1]">Dein Profil</span>
          </DropdownMenuItem>
          <DropdownMenuItem className="text-light text-base p-3.5 rounded-2xl transition-all duration-300 hover:bg-green-light/20 hover:text-green-light-200 focus:bg-green-light/20 focus:text-green-light-200 cursor-pointer">
            <LogOut className="mr-2 h-5 w-5" />
            <span className="font-lato font-semibold tracking-[0.1]">Abmelden</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  ),
}

export const UserProfileLoggedOut: Story = {
  render: () => (
    <div className="flex justify-end">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button type="button" className="group flex items-center gap-x-1">
            <Avatar>
              <AvatarFallback variant="guest">
                <UserRound className="w-5 h-5 stroke-2" />
              </AvatarFallback>
            </Avatar>
            <ChevronDown className="w-5 h-5 text-dark transition-transform duration-300 group-data-[state=open]:rotate-180" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-72 bg-dark border-none shadow-cards text-white text-sm pt-5 px-2"
        >
          <div className="border-b border-dark-800 mx-3 pb-4 mb-2">
            <p>Nicht angemeldet</p>
          </div>
          <DropdownMenuItem className="text-light text-base p-3.5 rounded-2xl transition-all duration-300 hover:bg-green-light/20 hover:text-green-light-200 focus:bg-green-light/20 focus:text-green-light-200 cursor-pointer">
            <LogIn className="mr-2 h-5 w-5" />
            <span className="font-lato font-semibold tracking-[0.1]">Anmelden</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  ),
}
