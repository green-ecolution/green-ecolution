import { lazy } from 'react'
import { Droplets, FolderClosed, PieChart, Route, TreeDeciduous, Trees } from 'lucide-react'
import Tree from '@/components/icons/Tree'
import type { ModuleManifest } from '@/modules/types'
import { irrigationSettingsSection } from './components/settings/settingsSection'
import { IRRIGATION } from './lib/targets'
import { irrigationMap } from './map'
import { irrigationPlanning } from './planning'
import { EVALUATION_READ, irrigationResources } from './resources'
import { treeSensorTarget } from './sensorTargets'

export const irrigationManifest: ModuleManifest = {
  key: IRRIGATION,
  order: 10,
  labelKey: 'irrigation:label',
  icon: Droplets,
  descriptionKey: 'irrigation:description',
  i18n: {
    namespaces: ['tree', 'treecluster', 'evaluation', 'irrigation'],
    // One `import()` per language literal, so each language gets its own chunk.
    load: async (lng) =>
      lng === 'en'
        ? (await import('./locales/en')).default
        : (await import('./locales/de')).default,
  },
  nav: {
    links: [
      {
        key: 'nav-green-spaces-clusters',
        to: '/treecluster',
        labelKey: 'irrigation:nav.clusters',
        icon: FolderClosed,
        section: 'greenspaces',
      },
      {
        key: 'nav-green-spaces-trees',
        to: '/trees',
        labelKey: 'irrigation:nav.trees',
        icon: Tree,
        section: 'greenspaces',
      },
      {
        key: 'nav-more-evaluation',
        to: '/evaluations',
        labelKey: 'irrigation:nav.evaluations',
        icon: PieChart,
        section: 'analysis',
      },
    ],
  },
  routePermissions: {
    '/trees': ['tree:read'],
    '/treecluster': ['tree_cluster:read'],
    '/evaluations': EVALUATION_READ,
  },
  resources: irrigationResources,
  roleTemplates: [
    { key: 'tree_care', nameKey: 'irrigation:roles.tree_care', icon: TreeDeciduous },
    { key: 'route_planning', nameKey: 'irrigation:roles.route_planning', icon: Route },
  ],
  dashboardTiles: [lazy(() => import('./components/dashboard/ClustersTile'))],
  statistics: {
    tree: {
      icon: TreeDeciduous,
      labelKey: 'irrigation:statistics.tree.label',
      chartLabelKey: 'irrigation:statistics.tree.chartLabel',
      to: '/treecluster',
      hero: {
        companionKind: 'cluster',
        companionKey: 'irrigation:statistics.hero.managedInGroups',
        linkLabelKey: 'irrigation:statistics.hero.viewAllGroupsLink',
      },
    },
    cluster: {
      icon: Trees,
      labelKey: 'irrigation:statistics.cluster.label',
      chartLabelKey: 'irrigation:statistics.cluster.chartLabel',
      subtitleKey: 'irrigation:statistics.cluster.subtitle',
      to: '/treecluster',
    },
  },
  map: irrigationMap,
  planning: irrigationPlanning,
  targets: [treeSensorTarget],
  settingsSection: irrigationSettingsSection,
  // Lazy like the other slots; the series exists for any soil-moisture sensor,
  // the tree only adds thresholds and watering events.
  sensorInsights: lazy(() => import('./components/sensor/SensorSoilMoistureChart')),
}

export default irrigationManifest
