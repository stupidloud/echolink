import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import Dashboard from '../views/Dashboard.vue'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(vi.fn())),
}))

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation(async (cmd) => {
      if (cmd === 'get_settings') {
        return { protocol: 'openai', baseUrl: '', apiKey: '', model: 'whisper-1' }
      }
      if (cmd === 'get_history') return []
      return null
    })
  })

  it('mounts and renders the title', async () => {
    const wrapper = mount(Dashboard)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('自然说话，完美书写')
  })

  it('shows waiting state initially', async () => {
    const wrapper = mount(Dashboard)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('等待语音输入')
  })

  it('computes stats from history', async () => {
    invoke.mockImplementation(async (cmd) => {
      if (cmd === 'get_settings') return { protocol: 'openai', baseUrl: '', apiKey: '', model: 'whisper-1' }
      if (cmd === 'get_history') return [
        { text: '你好世界' },
        { text: '测试' },
      ]
      return null
    })
    const wrapper = mount(Dashboard)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('6 字')
    expect(wrapper.text()).toContain('字')
  })
})
