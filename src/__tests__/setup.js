import { vi } from 'vitest'

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
