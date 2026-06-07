import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ApiSettings from '../views/ApiSettings.vue'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('ApiSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    invoke.mockImplementation(async (cmd) => {
      if (cmd === 'get_settings') {
        return { protocol: 'stepfun', baseUrl: 'https://api.stepfun.com', apiKey: '', model: 'stepaudio-2.5-asr' }
      }
      return null
    })
  })

  it('mounts and renders protocol selector', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    expect(wrapper.text()).toContain('Protocol')
  })

  it('loads settings on mount', async () => {
    mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    expect(invoke).toHaveBeenCalledWith('get_settings')
  })

  it('calls save_settings on save button click', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    const saveBtn = wrapper.find('.save-btn')
    expect(saveBtn.exists()).toBe(true)
    await saveBtn.trigger('click')
    await new Promise(r => setTimeout(r, 10))
    expect(invoke).toHaveBeenCalledWith('save_settings', expect.any(Object))
  })
})
