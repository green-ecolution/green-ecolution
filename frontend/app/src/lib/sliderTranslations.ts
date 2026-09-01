import type { TFunction } from 'i18next'

/** Splide's accessibility labels. `%s` is Splide's own placeholder, not i18next's. */
export function sliderLabels(t: TFunction<'navigation'>) {
  return {
    prev: t('slider.prev'),
    next: t('slider.next'),
    first: t('slider.first'),
    last: t('slider.last'),
    slideX: t('slider.slideX'),
    pageX: t('slider.pageX'),
    carousel: t('slider.carousel'),
    select: t('slider.select'),
    slide: t('slider.slide'),
    slideLabel: t('slider.slideLabel'),
  }
}
