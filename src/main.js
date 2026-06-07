import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

import { load } from '@tauri-apps/plugin-store'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'
import Database from '@tauri-apps/plugin-sql'
import { listen } from '@tauri-apps/api/event'

const app = createApp(App)
app.use(router)

let unlistenRecording = null

async function setupPlugins() {
  try {
    const [store, clipboard, db] = await Promise.all([
      load('settings.json'),
      Promise.resolve({ readText, writeText }),
      Database.load('sqlite:echolink.db'),
    ])

    app.provide('store', store)
    app.provide('clipboard', clipboard)
    app.provide('db', db)

    unlistenRecording = await listen('recording-state', (event) => {
      app.config.globalProperties.$isRecording = event.payload
    })
  } catch (e) {
    console.warn('Tauri plugins init failed, running in browser mode:', e)
  }
}

setupPlugins().then(() => {
  app.mount('#app')
})

export { unlistenRecording }
