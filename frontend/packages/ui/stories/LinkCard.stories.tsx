import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  LinkCard,
  LinkCardTitle,
  LinkCardDescription,
  LinkCardFooter,
} from '../src/components/ui/link-card'

const meta: Meta<typeof LinkCard> = {
  title: 'UI/LinkCard',
  component: LinkCard,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['dark', 'light', 'white'],
    },
    asChild: {
      control: 'boolean',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: (args) => (
    <LinkCard {...args}>
      <LinkCardTitle>Bäume verwalten</LinkCardTitle>
      <LinkCardDescription>
        Übersicht aller Bäume im System, Sensordaten einsehen und Bewässerungsstatus prüfen.
      </LinkCardDescription>
      <LinkCardFooter>Zur Übersicht</LinkCardFooter>
    </LinkCard>
  ),
  args: {
    variant: 'white',
  },
}

export const Dark: Story = {
  render: (args) => (
    <LinkCard {...args}>
      <LinkCardTitle>Bewässerungsgruppen</LinkCardTitle>
      <LinkCardDescription>
        Bäume in Gruppen organisieren und gemeinsam bewässern.
      </LinkCardDescription>
      <LinkCardFooter>Gruppen anzeigen</LinkCardFooter>
    </LinkCard>
  ),
  args: {
    variant: 'dark',
  },
}

export const Light: Story = {
  render: (args) => (
    <LinkCard {...args}>
      <LinkCardTitle>Einsatzpläne</LinkCardTitle>
      <LinkCardDescription>Bewässerungseinsätze planen und Routen optimieren.</LinkCardDescription>
      <LinkCardFooter>Pläne verwalten</LinkCardFooter>
    </LinkCard>
  ),
  args: {
    variant: 'light',
  },
}

export const WithLink: Story = {
  render: (args) => (
    <LinkCard {...args} asChild>
      <a href="/trees">
        <LinkCardTitle>Bäume</LinkCardTitle>
        <LinkCardDescription>Alle Bäume im System anzeigen und verwalten.</LinkCardDescription>
        <LinkCardFooter>Zur Baumübersicht</LinkCardFooter>
      </a>
    </LinkCard>
  ),
  args: {
    variant: 'white',
  },
}

export const WithoutArrow: Story = {
  render: (args) => (
    <LinkCard {...args}>
      <LinkCardTitle>Statistiken</LinkCardTitle>
      <LinkCardDescription>Übersicht über Bewässerungsstatistiken und Trends.</LinkCardDescription>
      <LinkCardFooter showArrow={false}>Mehr erfahren</LinkCardFooter>
    </LinkCard>
  ),
  args: {
    variant: 'white',
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="grid gap-6 md:grid-cols-3">
      <LinkCard variant="white">
        <LinkCardTitle>White Variant</LinkCardTitle>
        <LinkCardDescription>Standard-Variante mit weißem Hintergrund.</LinkCardDescription>
        <LinkCardFooter>Mehr erfahren</LinkCardFooter>
      </LinkCard>
      <LinkCard variant="light">
        <LinkCardTitle>Light Variant</LinkCardTitle>
        <LinkCardDescription>Hellgrüne Variante für Aktionen.</LinkCardDescription>
        <LinkCardFooter>Mehr erfahren</LinkCardFooter>
      </LinkCard>
      <LinkCard variant="dark">
        <LinkCardTitle>Dark Variant</LinkCardTitle>
        <LinkCardDescription>Dunkelgrüne Variante für wichtige Bereiche.</LinkCardDescription>
        <LinkCardFooter>Mehr erfahren</LinkCardFooter>
      </LinkCard>
    </div>
  ),
}

export const DashboardExample: Story = {
  render: () => (
    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      <LinkCard variant="dark" asChild>
        <a href="/trees">
          <LinkCardTitle>Alle Bäume</LinkCardTitle>
          <LinkCardDescription>
            Verwalten Sie alle Bäume im System und prüfen Sie deren Bewässerungsstatus.
          </LinkCardDescription>
          <LinkCardFooter>Zur Baumübersicht</LinkCardFooter>
        </a>
      </LinkCard>
      <LinkCard variant="light" asChild>
        <a href="/treecluster">
          <LinkCardTitle>Bewässerungsgruppen</LinkCardTitle>
          <LinkCardDescription>
            Organisieren Sie Bäume in Gruppen für effiziente Bewässerung.
          </LinkCardDescription>
          <LinkCardFooter>Gruppen verwalten</LinkCardFooter>
        </a>
      </LinkCard>
      <LinkCard variant="white" asChild>
        <a href="/watering-plans">
          <LinkCardTitle>Einsatzpläne</LinkCardTitle>
          <LinkCardDescription>
            Planen Sie Bewässerungseinsätze und optimieren Sie Routen.
          </LinkCardDescription>
          <LinkCardFooter>Pläne erstellen</LinkCardFooter>
        </a>
      </LinkCard>
    </div>
  ),
}
