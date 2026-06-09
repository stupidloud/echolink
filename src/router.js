import { createRouter, createWebHashHistory } from 'vue-router'
import Dashboard from './views/Dashboard.vue'
import ApiSettings from './views/ApiSettings.vue'
import History from './views/History.vue'
import OverlayIndicator from './views/OverlayIndicator.vue'

const routes = [
  { path: '/', redirect: '/dashboard' },
  { path: '/dashboard', component: Dashboard },
  { path: '/api-settings', component: ApiSettings },
  { path: '/history', component: History },
  { path: '/overlay', component: OverlayIndicator },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
