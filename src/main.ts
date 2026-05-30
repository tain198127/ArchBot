import { createApp } from 'vue'
import PrimeVue from 'primevue/config'
import { i18n } from './i18n'
import { pushLog } from './stores/log'
import App from './App.vue'
import '@fontsource/inter/400.css'
import '@fontsource/inter/500.css'
import '@fontsource/inter/600.css'
import '@fontsource/inter/700.css'
import '@fontsource/jetbrains-mono/400.css'
import './tailwind.css'

// Capture unhandled errors and rejected promises for the log panel.
window.addEventListener('error', (e) => {
  pushLog('error', 'global', `${e.message} (${e.filename}:${e.lineno})`)
})
window.addEventListener('unhandledrejection', (e) => {
  pushLog('error', 'global', `Unhandled rejection: ${String(e.reason)}`)
})

const app = createApp(App)
app.use(PrimeVue, { unstyled: true })
app.use(i18n)
app.mount('#app')
