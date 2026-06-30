import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },
  {
    path: '/games',
    name: 'games',
    component: () => import('@/views/GamesView.vue'),
  },
  {
    path: '/play/:gameId',
    name: 'play',
    component: () => import('@/views/PlayView.vue'),
    props: true,
  },
  // Engine demo: available in development only, not on public production routes.
  ...(import.meta.env.DEV
    ? [
        {
          path: '/demo',
          name: 'demo',
          component: () => import('@/views/DemoView.vue'),
        } satisfies RouteRecordRaw,
      ]
    : []),
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/NotFoundView.vue'),
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
