import { vi } from 'vitest'
import { config } from '@vue/test-utils'
import { i18n, setLocale } from '../i18n'

// Tests were written against the Chinese UI strings, so pin the test locale to zh
// (the app itself defaults to the system language). Installing i18n globally gives
// every mounted component access to $t without per-test wiring.
setLocale('zh')
config.global.plugins = [i18n]

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(),
  writeText: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() => Promise.resolve({
    get: vi.fn(),
    set: vi.fn(),
    save: vi.fn(),
  })),
}))

vi.mock('@tauri-apps/plugin-sql', () => ({
  default: {
    load: vi.fn(() => Promise.resolve({
      select: vi.fn(),
      execute: vi.fn(),
      close: vi.fn(),
    })),
  },
}))

const mockListen = vi.fn(() => Promise.resolve(vi.fn()))
vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args) => mockListen(...args),
}))

global.__TAURI__ = {
  event: {
    emit: vi.fn(),
    listen: vi.fn(),
  },
}
