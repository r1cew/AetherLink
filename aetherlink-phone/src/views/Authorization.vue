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
          <input
            v-model="jsonInput"
            placeholder='Вставьте JSON: {"server_id": "..."}'
            type="text"
          />
          <button
            @click="handleJsonLogin"
            :disabled="!jsonInput.trim() || loading"
          >
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
import { watch, onMounted, ref } from "vue";

const router = useRouter();
const {
  startQrScan,
  isJustConnected,
  active,
  profiles,
  servers,
  loading,
  autoConnect,
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

//Сделать нормально при фиксе ошибок-------(костыль)---------------------------------------------------

const jsonInput = ref("");

async function handleJsonLogin() {
  if (!jsonInput.value.trim()) return;

  try {
    // Пробуем распарсить JSON
    const jsonData = JSON.parse(jsonInput.value.trim());

    // Вызываем тот же метод autoConnect, что и для QR-сканера
    await autoConnect(JSON.stringify(jsonData));
    router.push("/main");
    // Очищаем поле после успешного входа
    jsonInput.value = "";
  } catch (error) {
    console.error("Ошибка парсинга JSON:", error);
    alert("Неверный формат JSON. Пожалуйста, проверьте данные.");
  }
}
</script>

<style scoped>
@import "../styles/authorization.css";
</style>
