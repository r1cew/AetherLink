<template>
  <div class="app-wrapper">
    <button class="paint-bucket" @click="toggleColorPicker">
      <PaintBucket />
    </button>
    <div class="page-content">
      <router-view />
    </div>
    <GlobalNotifications />
    <QrScanner v-if="isScanning" @success="autoConnect" @cancel="stopQrScan" />
    <ColorPickerPanel />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAetherLink } from "./composables/useAetherLink";
import { useNotification } from "./composables/useNotification";
import { useThemeCustomizer } from "./composables/useThemeCustomizer";
import QrScanner from "./views/QrScanner.vue";
import GlobalNotifications from "./views/components/GlobalNotifications.vue";
import ColorPickerPanel from "./views/components/ColorPickerPanel.vue";
import { PaintBucket } from "lucide-vue-next";

const { isScanning, loadServers, autoConnect, stopQrScan, discover } =
  useAetherLink();
const { success, error } = useNotification();
const { toggleColorPicker } = useThemeCustomizer();

onMounted(async () => {
  try {
    await discover();
    await loadServers();
    success("Серверы успешно загружены");
  } catch (err) {
    error("Ошибка загрузки серверов");
  }
});
</script>

<style>
.page-content {
  margin-top: 10vh;
}

.paint-bucket {
  position: absolute;
  margin-top: -2vh;
  margin-left: 12vw;
  background: none;
  border: none;
  color: var(--red);
  border-bottom: 1px solid var(--red);
  cursor: pointer;
  transition: all 0.2s ease;
}

.paint-bucket:active {
  filter: drop-shadow(0 0 20px rgba(255, 0, 17, 0.8));
  transform: scale(0.9);
}

.paint-bucket:hover {
  color: var(--accent2);
  border-bottom-color: var(--accent2);
}
</style>
