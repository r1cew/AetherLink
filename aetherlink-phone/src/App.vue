<template>
  <div class="app-wrapper">
    <div class="page-content">
      <router-view />
    </div>
    <GlobalNotifications />
    <QrScanner v-if="isScanning" @success="autoConnect" @cancel="stopQrScan" />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAetherLink } from "./composables/useAetherLink";
import { useNotification } from "./composables/useNotification";
import QrScanner from "./views/QrScanner.vue";
import GlobalNotifications from "./views/components/GlobalNotifications.vue";

const { isScanning, loadServers, autoConnect, stopQrScan, discover } =
  useAetherLink();
const { success, error } = useNotification();

onMounted(async () => {
  try {
    await loadServers();
    await discover();
    success("Серверы успешно загружены");
  } catch (err) {
    error("Ошибка загрузки серверов");
  }
});
</script>
