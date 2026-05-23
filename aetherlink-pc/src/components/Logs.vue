<template>
  <section>
    <h2>Лог</h2>
    <div class="log-box">
      <div v-for="(line, i) in logData" :key="i" class="log-line">
        {{ line }}
      </div>
      <div v-if="logData.length === 0" class="empty">Логи пока пусты...</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAetherLink } from "../composables/useAetherLink";
import "../assets/style.css";

const { logData, get_logs } = useAetherLink();

onMounted(async () => {
  await get_logs();
});
</script>
