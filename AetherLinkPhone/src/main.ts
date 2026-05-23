import { createApp } from "vue";
import App from "./App.vue";
import "./App.css"; // Если хочешь оставить глобальные стили
import router from "./router";

const app = createApp(App);

app.use(router);
app.mount("#app");
