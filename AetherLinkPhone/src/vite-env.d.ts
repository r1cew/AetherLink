/// <reference types="vite/client" />

// Это нужно, чтобы TypeScript понимал, что такое .vue файлы
declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
