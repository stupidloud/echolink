import { createI18n } from 'vue-i18n'
import zh from './locales/zh.js'
import en from './locales/en.js'

export const SUPPORTED = ['zh', 'en']
const STORAGE_KEY = 'echolink-lang'

// Resolve the startup locale:
//   1. an explicit choice the user saved before  ->  use it
//   2. otherwise follow the system / webview language  ->  zh* => zh, else en
//   3. fall back to en
function resolveLocale() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved && SUPPORTED.includes(saved)) return saved
  } catch {}
  const sys = (navigator.language || '').toLowerCase()
  if (sys.startsWith('zh')) return 'zh'
  return 'en'
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: resolveLocale(),
  fallbackLocale: 'en',
  messages: { zh, en },
})

// The active language code, resolved fresh from storage each call. Used by the
// overlay window to tag ASR requests: it runs in a separate webview from the main
// window, so reading localStorage (shared origin) reflects a just-changed choice
// even though its own in-memory i18n instance wouldn't. 'zh'/'en' double as the
// ISO-639-1 codes the transcription APIs expect.
export function currentLocale() {
  return resolveLocale()
}

// The Chinese script the system locale implies, for normalizing ASR output:
//   'Hans' (Simplified) | 'Hant' (Traditional) | '' (non-Chinese, no conversion).
// Derived from the RAW system/webview locale — the UI 中/EN toggle only chooses
// 'zh'/'en' and carries no script, so we read navigator.language directly here.
// TW/HK/MO regions and an explicit Hant subtag mean Traditional; any other zh* is
// Simplified. Sent alongside the ASR request so the backend can convert the result.
export function currentScript() {
  const sys = (navigator.language || '').toLowerCase()
  if (!sys.startsWith('zh')) return ''
  if (sys.includes('hant') || sys.includes('-tw') || sys.includes('-hk') || sys.includes('-mo')) return 'Hant'
  return 'Hans'
}

// Switch language at runtime and remember the choice across launches.
export function setLocale(lang) {
  if (!SUPPORTED.includes(lang)) return
  i18n.global.locale.value = lang
  try { localStorage.setItem(STORAGE_KEY, lang) } catch {}
  document.documentElement.lang = lang === 'zh' ? 'zh-CN' : 'en'
}
