/**
 * Container width at which the settings pages switch from a Drawer to their
 * two-pane master-detail layout.
 *
 * Measured on the container, not the viewport: the app sidebar is collapsible and
 * shifts the available width by ~260px independently of window size, so no viewport
 * breakpoint is correct for both sidebar states.
 *
 * The value is what the detail pane is worth having: the list column takes 280-300px
 * across the three pages and the gap 24px, leaving ~400px for the detail. That is
 * about the width the Drawer itself would give it, so from here up two panes are
 * strictly better than one pane plus a Drawer. Shared by all three settings pages so
 * they switch at the same felt width.
 */
export const TWO_PANE_MIN_WIDTH = 720
