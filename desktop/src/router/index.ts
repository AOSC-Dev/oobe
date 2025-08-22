import { createRouter, createWebHistory } from "vue-router";
import AbortView from "@/views/AbortView.vue";
import CompleteView from "@/views/CompleteView.vue";
import ErrorView from "@/views/ErrorView.vue";
import HostnameView from "@/views/HostnameView.vue";
import UserView from "@/views/UserView.vue";
import ConfirmView from "@/views/ConfirmView.vue";
import SwapFileView from "@/views/SwapFileView.vue";
import WelcomeView from "@/views/WelcomeView.vue";
import LocaleView from '@/views/LocaleView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: WelcomeView,
      meta: { steps: 0, next: "/users" },
    },
    {
      path: "/users",
      name: "users",
      component: UserView,
      meta: { steps: 1, next: "/hostname" },
    },
    {
      path: "/abort",
      name: "abort",
      props: (route) => ({ ...route.query, ...route.params }),
      component: AbortView,
    },
    {
      path: "/finish",
      name: "complete",
      component: CompleteView,
      meta: { steps: 4 },
    },
    {
      path: "/error/:message",
      name: "error",
      props: (route) => ({ ...route.query, ...route.params }),
      component: ErrorView,
    },
    {
      path: "/swapfile",
      name: "swapfile",
      component: SwapFileView,
      meta: { steps: 1, next: "/confirm" },
    },
    {
      path: "/locales",
      name: "locales",
      component: LocaleView,
      meta: { steps: 1, next: "/swapfile" },
    },
    {
      path: "/confirm",
      name: "confirm",
      component: ConfirmView,
      meta: { steps: 1, next: "/finish" },
    },
    {
      path: "/hostname",
      name: "hostname",
      component: HostnameView,
      meta: { steps: 1, next: "/locales" },
    },
  ],
});

export default router;
