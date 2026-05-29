<template>
  <div class="app-wrapper">
    <div class="page-content">
      <router-view />
    </div>

    <QrScanner v-if="isScanning" @success="autoConnect" @cancel="stopQrScan" />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAetherLink } from "./composables/useAetherLink";
import QrScanner from "./views/QrScanner.vue";

const { isScanning, loadServers, autoConnect, stopQrScan, discover } =
  useAetherLink();

onMounted(() => {
  loadServers();
  discover();
});
</script>
