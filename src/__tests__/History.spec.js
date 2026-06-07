import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import History from '../views/History.vue'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const mockRows = [
  { id: '1', timestamp: '2026-06-07 12:00', text: '你好世界', protocol: 'stepfun', target_app: '当前应用' },
  { id: '2', timestamp: '2026-06-07 12:01', text: '测试文字', protocol: 'openai', target_app: '当前应用' },
]

describe('History', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation(async (cmd) => {
      if (cmd === 'get_history') return mockRows
      if (cmd === 'delete_history') return null
      return null
    })
  })

  it('mounts and shows empty state when no data', async () => {
    invoke.mockImplementation(async (cmd) => {
      if (cmd === 'get_history') return []
      return null
    })
    const wrapper = mount(History)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('暂无历史记录')
  })

  it('renders history rows', async () => {
    const wrapper = mount(History)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('你好世界')
    expect(wrapper.text()).toContain('测试文字')
    expect(wrapper.text()).toContain('stepfun')
    expect(wrapper.text()).toContain('openai')
  })

  it('filters rows by search query', async () => {
    const wrapper = mount(History)
    await new Promise(r => setTimeout(r, 10))
    const input = wrapper.find('.search-input')
    await input.setValue('你好')
    expect(wrapper.text()).toContain('你好世界')
    expect(wrapper.text()).not.toContain('测试文字')
  })

  it('calls delete_history on remove', async () => {
    const wrapper = mount(History)
    await new Promise(r => setTimeout(r, 10))
    const deleteIcons = wrapper.findAll('.action-icon')
    await deleteIcons[1].trigger('click')
    await new Promise(r => setTimeout(r, 10))
    expect(invoke).toHaveBeenCalledWith('delete_history', { id: '1' })
  })
})
