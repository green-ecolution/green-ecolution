import { FolderClosed, PieChart, TreeDeciduous, Trees } from 'lucide-react'
import Tree from '@/components/icons/Tree'
import { ANY_READ } from '@/lib/auth/permissions'
import type { ModuleManifest } from '@/modules/types'
import ClustersTile from './components/dashboard/ClustersTile'
import { irrigationMap } from './map'

export const irrigationManifest: ModuleManifest = {
  key: 'irrigation',
  labelKey: 'common:modules.irrigation',
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
    '/evaluations': ANY_READ,
  },
  resources: [
    { key: 'tree', area: 'greenspaces' },
    { key: 'tree_cluster', area: 'greenspaces' },
    { key: 'watering_plan', area: 'operations' },
  ],
  dashboardTiles: [ClustersTile],
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
}
