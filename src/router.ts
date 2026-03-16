import { createRouter, createWebHistory } from 'vue-router'
import LoadingView from '@/views/LoadingView.vue'
import DashboardView from '@/views/DashboardView.vue'
import CategoriesView from '@/views/CategoriesView.vue'
import TransactionsView from '@/views/TransactionsView.vue'
import ImportView from '@/views/ImportView.vue'
import SettingsView from '@/views/SettingsView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: LoadingView },
    { path: '/dashboard', component: DashboardView },
    { path: '/categories', component: CategoriesView },
    { path: '/transactions', component: TransactionsView },
    { path: '/import', component: ImportView },
    { path: '/settings', component: SettingsView },
  ],
})

export default router