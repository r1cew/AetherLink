<template>
    <div class="app-wrapper">
        <div class="page-content">
            <router-view />
        </div>

        <QrScanner
            v-if="isScanning"
            @success="autoConnect"
            @cancel="stopQrScan"
        />
    </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useAetherLink } from "./composables/useAetherLink";
// Импортируем компонент сканера (проверь путь, если положил его в src/components)
import QrScanner from "./views/QrScanner.vue";

// Достаем флаг состояния сканирования и методы обработки результатов
const { isScanning, loadServers, autoConnect, stopQrScan } = useAetherLink();

onMounted(() => {
    loadServers();
});
</script>
