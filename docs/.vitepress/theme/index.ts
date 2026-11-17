import DefaultTheme from 'vitepress/theme'
import { h } from 'vue'
import LapFeatures from './components/LapFeatures.vue'
import './custom.css'

export default {
    extends: DefaultTheme,
    Layout: () => {
        return h(DefaultTheme.Layout, null, {
            'home-hero-actions-after': () => h('p', { class: 'download-note' }, 'Free to use. No subscription or recurring fees.'),
            'home-features-before': () => h(LapFeatures)
        })
    }
}
