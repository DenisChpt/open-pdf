import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },
    {
      path: "/merge",
      name: "merge",
      component: () => import("../views/MergeView.vue"),
    },
    {
      path: "/split",
      name: "split",
      component: () => import("../views/SplitView.vue"),
    },
    {
      path: "/compress",
      name: "compress",
      component: () => import("../views/CompressView.vue"),
    },
    {
      path: "/organize",
      name: "organize",
      component: () => import("../views/OrganizeView.vue"),
    },
    {
      path: "/convert",
      name: "convert",
      component: () => import("../views/ConvertView.vue"),
    },
    {
      path: "/sign",
      name: "sign",
      component: () => import("../views/SignatureView.vue"),
    },
  ],
});

export default router;
