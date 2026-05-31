<template>
  <div v-if="devStatus && devStatus.is_dev === true">
    <select v-model="selectedProfileId">
      <option v-for="profile in profiles" :key="profile.id" :value="profile.id">
        {{ profile.name }}
      </option>
    </select>
    <button
      @click="runProfile(selectedProfileId, profileName)"
      :disabled="!selectedProfileId || loading"
    >
      {{ loading ? "Запуск..." : "Запустить" }}
    </button>
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
import { ref, onMounted, computed } from "vue";
const { profiles, runProfile, devStatus, checkDev, loading } = useAetherLink();

const selectedProfileId = ref<string>("");

const profileName = computed<string>(() => {
  const profilesList = profiles.value;
  if (!profilesList) return "";
  return profilesList.find((p) => p.id === selectedProfileId.value)?.name ?? "";
});

onMounted(() => {
  checkDev();
});
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

select {
  padding: 10px;
  cursor: pointer;
  padding: 10px;
  cursor: pointer;
  border-radius: 12px;
  border: 2px solid var(--accent);
  background: none;
  color: var(--text);
}

select option {
  background-color: #1a1a1a;
  color: var(--text);
}

select:focus {
  outline: none;
  background-color: #1a1a1a;
}

button {
  background: var(--red);
  color: white;
  font-weight: 600;
  border-radius: 12px;
  border: none;
  transition: 0.3s;
}

button:disabled {
  background: var(--text2);
  transition: 0.3s;
}

.devOffline {
  display: flex;
  flex-direction: column;
  justify-content: center;
  color: var(--text2);
  text-align: center;
  gap: 30px;
}
</style>
