<template>
  <div v-if="devStatus && devStatus === true">
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
    Дев статус не найден (Заглушка пока что, мейби там ошибка. Чекай консоль)<br />
    <br />
    Мейби если ошибки не будет будет отображать инфу с запуском команд
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
