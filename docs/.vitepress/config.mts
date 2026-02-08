import { defineConfig } from 'vitepress'

export default defineConfig({
    base: '/lap/',
    title: "Lap",
    description: "Local-first, AI-powered photo manager",
    head: [['link', { rel: 'icon', href: '/logo.png' }]],
    themeConfig: {
        logo: '/logo.png',
        nav: [
            { text: 'Home', link: '/' },
            { text: 'Guide', link: '/guide/introduction' },
            { text: 'Download', link: 'https://github.com/julyx10/lap/releases' }
        ],
        sidebar: [
            {
                text: 'Guide',
                items: [
                    { text: 'Introduction', link: '/guide/introduction' },
                    { text: 'Getting Started', link: '/guide/getting-started' }
                ]
            }
        ],
        socialLinks: [
            { icon: 'github', link: 'https://github.com/julyx10/lap' }
        ],
        footer: {
            message: 'Released under the GPL-3.0 License.',
            copyright: 'Copyright © 2024 Lap Contributors'
        }
    }
})
