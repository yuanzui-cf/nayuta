import type { Icon } from './icon';

export interface Link {
  text?: string;
  url?: string;
  href?: string;
  icon?: Icon;
  target?: string;
  ariaLabel?: string;
  iconOnly?: boolean;
}

export interface Config {
  // Site title
  title: string;
  // Author name
  author: string;
  // Author avatar
  avatar: string;
  // Brief author description
  description?: string;
  // Site URL
  site_url?: string;
  // Website start/launch date for running time calculation (e.g. '2024-01-01')
  since?: string | Date;
  site_start_date?: string | Date;

  // Theme name in src/styles/themes/ without .css extension (e.g. 'nayuta-aqua')
  theme?: string;

  // Profile Card links
  links?: Link[];
}
