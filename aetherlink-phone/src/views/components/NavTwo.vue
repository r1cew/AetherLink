<template>
  <div>
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
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";
import { ref } from "vue";

const { profiles, runProfile } = useAetherLink();
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
