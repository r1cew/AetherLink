<template>
  <section class="main-page">
    <div class="name-block">
      <h1>AetherLink</h1>
    </div>
    <div class="status-block">
      <span class="status-ok" style="cursor: pointer" @click="logout">
        <ArrowBigLeft class="arrow" />
        {{ isJustConnected ? "Подключен" : "Не подключен" }}</span
      >
    </div>

    <div class="navigation-block">
      <button
        @click="nav_page = 1"
        :class="nav_page == 1 ? 'nav-btn active-btn' : 'nav-btn'"
      >
        Базовые
      </button>
      <button
        @click="nav_page = 2"
        :class="nav_page == 2 ? 'nav-btn active-btn' : 'nav-btn'"
      >
        Запустить
      </button>
      <button
        @click="nav_page = 3"
        :class="nav_page == 3 ? 'nav-btn active-btn' : 'nav-btn'"
      >
        Добавить
      </button>
    </div>
    <div class="components-block">
      <NavOne v-if="nav_page == 1" />
      <NavTwo v-if="nav_page == 2" />
      <NavThree v-if="nav_page == 3" />
    </div>
  </section>
</template>

<script setup>
import { useAetherLink } from "../composables/useAetherLink";
import { useRouter } from "vue-router";
import { Store } from "@tauri-apps/plugin-store";
import NavOne from "./components/NavOne.vue";
import NavTwo from "./components/NavTwo.vue";
import NavThree from "./components/NavThree.vue";
import { ref, onMounted } from "vue";
import leave from "../assets/leave.png";
import { ArrowBigLeft } from "lucide-vue-next";

const nav_page = ref(1);
const router = useRouter();
const {
  active,
  profiles,
  resetConnection,
  isJustConnected,
  devStatus,
  loadServers,
  servers,
  stopQrScan,
  loadProfiles,
} = useAetherLink();

async function logout() {
  await resetConnection();
  router.push("/");
}

onMounted(() => {
  loadProfiles(active.value);
  console.log(devStatus.value);
});
</script>

<style scoped>
@import "../styles/main.css";
</style>
