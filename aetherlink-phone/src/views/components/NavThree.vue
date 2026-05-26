<template>
  <div v-if="devStatus && devStatus.is_dev === true">
    <div class="input-block">
      <input v-model="newTask.name" placeholder="Название*" />
      <button @click="createProfile">+</button>
    </div>
    <div>
      <input
        v-model="newTask.description"
        class="description"
        placeholder="Описание"
      />
      <input v-model="newTask.command" class="command" placeholder="Команда" />
    </div>
    <select v-model="newTask.type">
      <option value="run_bat">Запуск bat*</option>
      <option value="run_exe">Запуск программы*</option>
      <option value="power_shell">PowerShell*</option>
    </select>
  </div>
  <div v-else>
    <div class="devOffline">
      <span>Режим Dev не активирован.</span>
      <span>Включите Dev режим на ПК версии и перезайдите в приложение.</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";
import { onMounted } from "vue";

const { newTask, devStatus, checkDev, createProfile } = useAetherLink();

onMounted(() => {
  checkDev();
});
</script>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

select,
input {
  padding: 10px;
  cursor: pointer;
  padding: 10px;
  cursor: pointer;
  border-radius: 12px;
  border: 2px solid rgba(216, 50, 60, 0.3);
  background: none;
  color: var(--red);
}

button {
  background: var(--red);
  color: white;
  font-weight: 600;
  border-top-right-radius: 12px;
  border-bottom-right-radius: 12px;
  border: none;
  cursor: pointer;
}

input {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}

.description,
.command {
  border-radius: 12px;
}

select option {
  background-color: #1a1a1a;
  color: var(--red);
}

select:focus {
  outline: none;
  background-color: #1a1a1a;
}

.input-block {
  display: flex;
  justify-content: center;
  flex-direction: row;
  gap: 0;
}

.history {
  display: flex;
  align-items: center;
}

.history div {
  display: flex;
  gap: 10px;
}

.history div button {
  width: 75vw;
}

.input-block input {
  width: 70vw;
}

.input-block button {
  width: 20vw;
}

.devOffline {
  display: flex;
  flex-direction: column;
  justify-content: center;
  color: var(--text2);
  text-align: center;
  gap: 30px;
}

.input-block input,
.input-block button {
  height: 40px;
}

.input-block input {
  width: 70vw;
}

.input-block button {
  width: 20vw;
}
</style>
