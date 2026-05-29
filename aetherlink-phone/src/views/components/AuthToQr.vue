<template>
  <div class="qr-auth-section">
    <div class="btns">
      <input v-model="phoneName" placeholder="Имя для текущего устройства" />
      <input v-model="pcName" placeholder="Название ПК (Удобное вам)" />
      <button @click="startQrScan()">Войти</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";

const { startQrScan, phoneName, pcName } = useAetherLink();
</script>

<style scoped>
.qr-auth-section {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.btns {
  margin-top: 0;
  display: flex;
  justify-content: center;
  flex-direction: column;
  gap: 20px;
}

.btns input {
  background: none;
  color: var(--text);
  border: none;
  border-bottom: 1px solid var(--text);
  padding: 10px;
}

.btns input:focus {
  border-bottom: 1px solid var(--red);
}

.btns button {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 296px;
  height: 50px;
  padding: 0;
  margin: 0;
  font-family: inherit;
  font-size: inherit;
  cursor: pointer;
  transition: background-color 0.2s ease;
  background: none;
  color: var(--text);
  border: 2px solid var(--red);
  border-radius: 13px;
}

.btns button:active {
  background: var(--red);
}

.btns button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btns select {
  width: 296px;
  height: 50px;
  padding: 0 16px;
  margin: 0;
  font-family: inherit;
  font-size: 16px;
  cursor: pointer;
  transition: background-color 0.2s ease;
  background: none;
  background-color: var(--bg);
  color: var(--text);
  border: 2px solid var(--red);
  border-radius: 13px;
  text-align: center;
  text-align-last: center;
  appearance: none;
}

.btns select option {
  text-align: left;
  text-align-last: left;
}

.json-input-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.json-input-group input {
  width: 280px;
  height: 40px;
  padding: 0 12px;
  font-family: inherit;
  font-size: 14px;
  background: none;
  background-color: var(--bg);
  color: var(--text);
  border: 2px solid var(--red);
  border-radius: 13px;
}

.json-input-group button {
  width: 296px;
  height: 50px;
  padding: 0;
  font-family: inherit;
  font-size: inherit;
  cursor: pointer;
  transition: background-color 0.2s ease;
  background: none;
  color: var(--text);
  border: 2px solid var(--red);
  border-radius: 13px;
}

.json-input-group button:active {
  background: var(--red);
}

.json-input-group button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
