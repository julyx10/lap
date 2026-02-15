import DefaultTheme from 'vitepress/theme'
import Fireworks from './components/Fireworks.vue'
import './custom.css'

export default {
    extends: DefaultTheme,
    enhanceApp({ app }) {
        app.component('Fireworks', Fireworks)
    }
}
