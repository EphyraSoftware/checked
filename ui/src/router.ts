import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", component: () => import("./pages/HomePage.vue") },
    { path: "/search", component: () => import("./pages/SearchPage.vue") },
    {
      path: "/assets",
      component: () => import("./pages/MyAssetSignaturesPage.vue"),
    },
    { path: "/settings", component: () => import("./pages/SettingsPage.vue") },
    { path: "/about", component: () => import("./pages/AboutPage.vue") },
  ],
});

export default router;
