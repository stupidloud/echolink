import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import App from '../App.vue'

const mockListen = vi.fn()
vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args) => mockListen(...args),
}))

describe('App', () => {
  beforeEach(() => {
    mockListen.mockReset()
    mockListen.mockImplementation(() => Promise.resolve(vi.fn()))
  })

  async function mountApp() {
    const wrapper = mount(App, {
      global: {
        stubs: ['router-link', 'router-view'],
      },
    })
    await new Promise(r => setTimeout(r, 10))
    return wrapper
  }

  it('shows idle status text', async () => {
    const wrapper = await mountApp()
    expect(wrapper.text()).toContain('按住 Right Alt 开始语音输入')
  })

  it('updates status text and adds recording class on recording-state:true', async () => {
    const wrapper = await mountApp()
    const handler = mockListen.mock.calls.find(c => c[0] === 'recording-state')?.[1]
    expect(handler).toBeDefined()
    handler({ payload: true })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('正在录音...')
    expect(wrapper.find('.floating-status').classes()).toContain('recording')
  })

  it('removes recording class on recording-state:false', async () => {
    const wrapper = await mountApp()
    const handler = mockListen.mock.calls.find(c => c[0] === 'recording-state')?.[1]
    handler({ payload: true })
    await wrapper.vm.$nextTick()
    handler({ payload: false })
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).toContain('按住 Right Alt 开始语音输入')
    expect(wrapper.find('.floating-status').classes()).not.toContain('recording')
  })

  it('updates bar heights on audio-level event', async () => {
    const wrapper = await mountApp()
    const handler = mockListen.mock.calls.find(c => c[0] === 'audio-level')?.[1]
    expect(handler).toBeDefined()
    handler({ payload: 0.5 })
    await wrapper.vm.$nextTick()
    const bars = wrapper.findAll('.bar')
    for (const bar of bars) {
      const h = parseFloat(bar.attributes('style')?.match(/height:\s*([\d.]+)px/)?.[1] || '0')
      expect(h).toBeGreaterThanOrEqual(4)
      expect(h).toBeLessThanOrEqual(24)
    }
  })

  it('cleans up listeners on unmount', async () => {
    const unlisten1 = vi.fn()
    const unlisten2 = vi.fn()
    mockListen.mockReturnValueOnce(Promise.resolve(unlisten1))
    mockListen.mockReturnValueOnce(Promise.resolve(unlisten2))
    const wrapper = await mountApp()
    wrapper.unmount()
    expect(unlisten1).toHaveBeenCalled()
    expect(unlisten2).toHaveBeenCalled()
  })
})
