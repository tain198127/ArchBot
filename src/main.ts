import { createApp } from 'vue'
import PrimeVue from 'primevue/config'
import { i18n } from './i18n'
import App from './App.vue'
import '@fontsource/inter/400.css'
import '@fontsource/inter/500.css'
import '@fontsource/inter/600.css'
import '@fontsource/jetbrains-mono/400.css'
import './tailwind.css'

const app = createApp(App)
app.use(PrimeVue, { unstyled: true })
app.use(i18n)
app.mount('#app')
