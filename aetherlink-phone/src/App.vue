<template>
  <div class="app-wrapper">
    <div class="page-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "./composables/useAetherLink";
import { onMounted } from "vue";

const { loadServers } = useAetherLink();

onMounted(() => {
  loadServers();
});
</script>
