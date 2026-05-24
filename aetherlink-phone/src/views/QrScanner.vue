<template>
    <div class="web-scanner-overlay">
        <div class="scanner-card">
            <h3>Сканирование QR-кода</h3>
            <p class="scanner-hint">
                Наведите камеру на QR-код в приложении на ПК
            </p>

            <div id="qr-reader-container"></div>

            <button @click="handleCancel" class="cancel-scan-btn">
                Отмена
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { Html5Qrcode } from "html5-qrcode";

const emit = defineEmits<{
    (e: "success", text: string): void;
    (e: "cancel"): void;
}>();

let html5QrcodeScanner: Html5Qrcode | null = null;
const scannerId = "qr-reader-container";

onMounted(async () => {
    // Инициализируем сканер внутри созданного div
    html5QrcodeScanner = new Html5Qrcode(scannerId);

    const config = {
        fps: 15, // Частота кадров
        qrbox: { width: 250, height: 250 }, // Рамка фокуса
    };

    try {
        // Запускаем заднюю камеру (environment)
        await html5QrcodeScanner.start(
            { facingMode: "environment" },
            config,
            (decodedText) => {
                // Успешно отсканировано!
                emit("success", decodedText);
                stopScanner();
            },
            () => {
                // Сюда сыплется лог каждого кадра, если код не найден. Оставляем пустым.
            },
        );
    } catch (err) {
        console.error("Не удалось запустить камеру во Vue:", err);
        alert("Ошибка доступа к камере. Проверьте разрешения приложения.");
        emit("cancel");
    }
});

async function stopScanner() {
    if (html5QrcodeScanner && html5QrcodeScanner.isScanning) {
        try {
            await html5QrcodeScanner.stop();
        } catch (e) {
            console.error("Ошибка остановки камеры:", e);
        }
    }
}

function handleCancel() {
    stopScanner().then(() => {
        emit("cancel");
    });
}

onUnmounted(() => {
    stopScanner();
});
</script>

<style scoped>
.web-scanner-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
    padding: 20px;
    box-sizing: border-box;
}

.scanner-card {
    background: #1e1e1e;
    border: 2px solid var(--red, #ff4a4a);
    border-radius: 16px;
    padding: 20px;
    width: 100%;
    max-width: 400px;
    text-align: center;
    color: #fff;
}

.scanner-hint {
    font-size: 14px;
    color: #aaa;
    margin-bottom: 20px;
}

#qr-reader-container {
    width: 100%;
    border-radius: 12px;
    overflow: hidden;
    background: #000;
    margin-bottom: 20px;
}

/* Стилизация внутренних элементов html5-qrcode, чтобы убрать лишний мусор */
#qr-reader-container video {
    width: 100% !important;
    height: auto !important;
    border-radius: 12px;
}

.cancel-scan-btn {
    width: 100%;
    height: 45px;
    background: none;
    border: 2px solid #555;
    color: #fff;
    border-radius: 10px;
    cursor: pointer;
    font-size: 16px;
    transition: all 0.2s;
}

.cancel-scan-btn:active {
    background: #333;
}
</style>
