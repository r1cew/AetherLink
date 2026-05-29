<template>
  <section class="auth-page">
    <h1>AetherLink</h1>
    <div class="status">
      <div>
        <span class="status-default">{{
          isJustConnected ? "Подключен" : "Подключитесь к ПК"
        }}</span>
        <p>
          {{
            servers.length > 0
              ? "Устройства найдены!"
              : "Нет доступных устройств"
          }}
        </p>
        <div class="navigation-block">
          <button
            @click="nav_page = 1"
            :class="nav_page == 1 ? 'nav-btn active-btn' : 'nav-btn'"
          >
            Войти по QR
          </button>
          <button
            @click="nav_page = 2"
            :class="nav_page == 2 ? 'nav-btn active-btn' : 'nav-btn'"
          >
            Устройства
          </button>
        </div>

        <AuthToQr
          v-if="nav_page === 1"
          :servers="servers"
          :loading="loading"
          :jsonAuth="jsonAuth"
        />

        <AuthToSaveDevices
          v-if="nav_page === 2"
          :profiles="profiles"
          :active="active"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useAetherLink } from "../composables/useAetherLink";
import { ref } from "vue";
import AuthToQr from "./components/AuthToQr.vue";
import AuthToSaveDevices from "./components/AuthToSaveDevices.vue";

const nav_page = ref(1);

const { isJustConnected, active, profiles, servers, loading, jsonAuth } =
  useAetherLink();
</script>

<style scoped>
@import "../styles/authorization.css";

.navigation-block {
  display: flex !important;
  flex-direction: row !important;
  gap: 50px;
  justify-content: center;
  align-items: center;
}
.nav-btn {
  background: none;
  color: var(--text);
  border: none;
  cursor: pointer;
  padding-bottom: 2px;
  font-size: 16px;
}
.active-btn {
  border-bottom: 2px solid var(--red);
  transition: 0.3s;
}

/* Мобильные улучшения */
@media (max-width: 480px) {
  .navigation-block {
    gap: 30px !important;
  }

  .nav-btn {
    font-size: 14px !important;
  }
}
</style>
