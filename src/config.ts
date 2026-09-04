import type { Config } from './types/config';

const config: Config = {
  title: '✍️ Kani Nayuta',
  author: 'Kani Nayuta',
  avatar: '/assets/images/avatar.jpg',
  description:
    'Genius Light Novelist | Author of "The Landscape Series" | Operating principle: 100% Love for Itsuki Hashima',
  site_url: 'https://nayuta.kani.dev',
  theme: 'nayuta',
  links: [
    {
      content: {
        text: 'Posts',
      },
      url: '/posts',
    },
    {
      content: {
        text: 'Friends',
      },
      url: '/friend',
    },
  ],
};

export default config;
