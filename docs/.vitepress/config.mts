import { defineConfig } from 'vitepress'

export default defineConfig({
    base: '/lap/',
    title: "Lap",
    description: "Local-first, AI-powered photo manager",
    head: [
        ['link', { rel: 'icon', href: '/logo.png' }],
        [
            'script',
            { async: '', src: 'https://www.googletagmanager.com/gtag/js?id=G-SVT0K4C2ET' }
        ],
        [
            'script',
            {},
            `window.dataLayer = window.dataLayer || [];
             function gtag(){dataLayer.push(arguments);}
             gtag('js', new Date());
             gtag('config', 'G-SVT0K4C2ET');`
        ]
    ],
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
            },
            {
                text: 'Release Notes',
                items: [
                    { text: 'v0.1.6', link: '/guide/release-notes/v0.1.6' },
                    { text: 'v0.1.5', link: '/guide/release-notes/v0.1.5' },
                    { text: 'v0.1.4', link: '/guide/release-notes/v0.1.4' },
                    { text: 'v0.1.3', link: '/guide/release-notes/v0.1.3' },
                    { text: 'v0.1.2', link: '/guide/release-notes/v0.1.2' },
                    { text: 'v0.1.1', link: '/guide/release-notes/v0.1.1' },
                    { text: 'v0.1.0', link: '/guide/release-notes/v0.1.0' }
                ]
            }
        ],
        socialLinks: [
            { icon: 'github', link: 'https://github.com/julyx10/lap' }
        ],
        footer: {
            message: 'Released under the GPL-3.0 License.',
            copyright: 'Copyright © 2026 Lap Contributors'
        }
    }
})
