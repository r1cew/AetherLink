// composables/useQRPairing.ts
import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { UseLoggerReturn } from "./useLogger";

export function useQRPairing(logger: UseLoggerReturn) {
  const qrData = ref("");
  const timeLeft = ref(120);
  const timerActive = ref(false);
  let timerInterval: number | null = null;

  const formatTime = (seconds: number): string => {
    return seconds.toString();
  };

  const stopTimer = () => {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
      qrData.value = "";
    }
    timerActive.value = false;
  };

  const startTimer = () => {
    if (timerActive.value) return;
    if (timeLeft.value <= 0) {
      return;
    }

    timerActive.value = true;
    timerInterval = window.setInterval(() => {
      if (timeLeft.value > 0) {
        timeLeft.value--;
      } else {
        stopTimer();
      }
    }, 1000);
  };

  async function generateQR() {
    try {
      qrData.value = await invoke("generate_pairing_qr");
      await logger.addLog(`QR код сгенерирован`);
      timeLeft.value = 120;
      stopTimer();
      startTimer();
    } catch (e) {
      await logger.addLog(`Ошибка генерации QR: ${e}`);
      console.error("QR generation error:", e);
    }
  }

  function resetQR() {
    stopTimer();
    qrData.value = "";
    timeLeft.value = 120;
  }

  onUnmounted(() => {
    stopTimer();
  });

  return {
    qrData,
    timeLeft,
    timerActive,
    formatTime,
    generateQR,
    resetQR,
  };
}

export type UseQRPairingReturn = ReturnType<typeof useQRPairing>;
