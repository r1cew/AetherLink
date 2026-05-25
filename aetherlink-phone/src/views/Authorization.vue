<template>
  <section class="auth-page">
    <h1>AetherLink</h1>
    <div class="status">
      <div>
        <span class="status-default">{{
          isJustConnected ? "Подключен" : "Не подключен"
        }}</span>
        <p>
          {{
            servers.length > 0
              ? "Устройство найдено!"
              : "Устройство не распознано"
          }}
        </p>
        <p>Ожидаем авторизацию</p>
        <div class="btns">
          <button @click="startQrScan">Войти</button>
        </div>
        <div class="json-input-group">
          <input v-model="jsonAuth" placeholder="Вставьте JSON" type="text" />
          <button @click="JsonLogin" :disabled="!jsonAuth.trim() || loading">
            {{ loading ? "Подключение..." : "Войти по JSON" }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useAetherLink } from "../composables/useAetherLink";
import { useRouter } from "vue-router";
import { watch, onMounted } from "vue";

const router = useRouter();
const {
  startQrScan,
  isJustConnected,
  active,
  profiles,
  servers,
  loading,
  jsonAuth,
  JsonLogin,
} = useAetherLink();

console.log(servers);

onMounted(() => {
  if (active.value || profiles.value.length > 0) {
    console.log("Найдены сохраненные данные, переход на /main");
    router.push("/main");
  }
});

watch(isJustConnected, (connected) => {
  if (connected) {
    router.push("/main");
  }
});
</script>

<style scoped>
@import "../styles/authorization.css";
</style>
