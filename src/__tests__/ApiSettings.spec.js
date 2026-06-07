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
      if (cmd === 'verify_connection') return 'OK'
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
    await wrapper.find('.save-btn').trigger('click')
    await new Promise(r => setTimeout(r, 10))
    expect(invoke).toHaveBeenCalledWith('save_settings', expect.any(Object))
  })

  it('calls verify_connection on test button click', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    await wrapper.find('.test-btn').trigger('click')
    await new Promise(r => setTimeout(r, 10))
    expect(invoke).toHaveBeenCalledWith('verify_connection', expect.any(Object))
  })

  it('toggles password visibility on eye icon click', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    const input = wrapper.find('input[type="password"]')
    expect(input.exists()).toBe(true)
    await wrapper.find('.eye-icon').trigger('click')
    expect(wrapper.find('input[type="text"]').exists()).toBe(true)
  })

  it('renders model options from currentModels', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    const modelSelect = wrapper.findAll('select.form-select')[1]
    const options = modelSelect.findAll('option')
    const texts = options.map(o => o.text())
    expect(texts).toContain('stepaudio-2.5-asr')
    expect(texts).toContain('stepaudio-2-asr-pro')
  })

  it('updates model options when protocol changes', async () => {
    const wrapper = mount(ApiSettings)
    await new Promise(r => setTimeout(r, 10))
    const selects = wrapper.findAll('select.form-select')
    const protocolSelect = selects[0]
    const modelSelect = selects[1]

    await protocolSelect.setValue('openai')

    const options = modelSelect.findAll('option')
    const texts = options.map(o => o.text())
    expect(texts).toContain('whisper-1')
    expect(texts).not.toContain('stepaudio-2.5-asr')
  })
})
