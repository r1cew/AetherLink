import { createRouter, createWebHistory } from "vue-router";
import HomePage from "../views/HomePage.vue";
import TasksPage from "../views/TasksPage.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomePage,
    },
    {
      path: "/tasks",
      name: "tasks",
      component: TasksPage,
    },
  ],
});

export default router;
