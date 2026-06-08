import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

import { load } from '@tauri-apps/plugin-store'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'
import Database from '@tauri-apps/plugin-sql'
import { listen } from '@tauri-apps/api/event'
import { attachConsole } from '@tauri-apps/plugin-log'

const app = createApp(App)
app.use(router)

let unlistenRecording = null

async function setupPlugins() {
  let store = null
  let clipboard = null
  let db = null

  try {
    await attachConsole()
  } catch (e) {
    console.warn('[init] attachConsole failed:', e)
  }

  try {
    store = await load('settings.json')
  } catch (e) {
    console.warn('[init] store failed:', e)
  }

  try {
    clipboard = { readText, writeText }
  } catch (e) {
    console.warn('[init] clipboard failed:', e)
  }

  try {
    db = await Database.load('sqlite:echolink.db')
  } catch (e) {
    console.warn('[init] sql failed:', e)
  }

  if (store) app.provide('store', store)
  if (clipboard) app.provide('clipboard', clipboard)
  if (db) app.provide('db', db)

  try {
    unlistenRecording = await listen('recording-state', (event) => {
      console.log('[event] recording-state →', event.payload)
      app.config.globalProperties.$isRecording = event.payload
    })
    console.log('[init] event listener registered OK')
  } catch (e) {
    console.warn('[init] event listen failed:', e)
  }

  try {
    await listen('rdev-status', (event) => {
      console.log('[rdev]', event.payload)
    })
  } catch {}
}

setupPlugins().then(() => {
  app.mount('#app')
})

export { unlistenRecording }
