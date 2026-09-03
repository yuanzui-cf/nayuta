import type { Icon } from './icon';

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

  // Theme name in src/styles/themes/ without .css extension (e.g. 'nayuta-aqua')
  theme?: string;

  // Links to display in the header
  links?: Link[];
}

interface TextLink {
  text: string;
  icon?: Icon;
}

interface IconLink {
  icon: Icon;
}

export interface Link {
  content: TextLink | IconLink;
  url: string;
}
