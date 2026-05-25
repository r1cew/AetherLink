<template>
  <div v-if="devStatus && devStatus.is_dev === true">
    <select v-model="selectedProfileId">
      <option v-for="profile in profiles" :key="profile.id" :value="profile.id">
        {{ profile.name }}
      </option>
    </select>
    <button
      @click="runProfile(selectedProfileId)"
      :disabled="!selectedProfileId"
    >
      Запустить
    </button>
  </div>
  <div v-else>
    Режим Developer не активирован. Запуск профилей доступен только в режиме
    Developer.<br />
    Переключите ПК в режим Developer в настройках.<br />
    <br />
    Текущий статус: {{ devStatus }}
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";
import { ref } from "vue";

const { profiles, runProfile, devStatus } = useAetherLink();

const selectedProfileId = ref<string>("");
</script>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 50px;
}

button,
select {
  padding: 10px;
  cursor: pointer;
}
</style>
