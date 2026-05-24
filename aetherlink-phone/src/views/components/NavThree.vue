<template>
  <div>
    <div class="input-block">
      <input placeholder="Название*" />
      <button>+</button>
    </div>
    <div><input placeholder="Описание" /></div>
    <select v-model="newTask.type">
      <option value="run_bat">Запуск bat*</option>
      <option value="run_exe">Запуск программы*</option>
      <option value="power_shell">PowerShell*</option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";

const { newTask } = useAetherLink();
</script>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

button,
select,
input {
  padding: 10px;
  cursor: pointer;
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
</style>
