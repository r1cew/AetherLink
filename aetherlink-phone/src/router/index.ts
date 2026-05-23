import { createRouter, createWebHistory } from "vue-router";
import Authorization from "../views/Authorization.vue";
import MainPage from "../views/MainPage.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "auth",
      component: Authorization,
    },
    {
      path: "/main",
      name: "main",
      component: MainPage,
    },
  ],
});

export default router;
