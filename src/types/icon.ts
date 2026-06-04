/**
 * SVG Icon type
 */
export interface SVGIcon {
  url: string;
}

/**
 * Font icon support
 */
export interface FontIcon {
  class: string;
}

/**
 * Icon type.
 */
export type Icon = SVGIcon | FontIcon;
