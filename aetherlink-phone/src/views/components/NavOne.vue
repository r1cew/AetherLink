<template>
  <div>
    <button @click="safe('shutdown')">Shutdow</button>
    <button @click="safe('sleep')">Sleep</button>
    <button @click="safe('lock')">LockScreen</button>
    <div class="double-btns">
      <button @click="safe('volume_down')" class="btn1">Volume -</button>
      <button @click="safe('volume_up')" class="btn2">Volume +</button>
    </div>
    <div class="media-btns">
      <button @click="safe('media_prev')">Prev</button>
      <button @click="safe('media_pause')">Stop</button>
      <button @click="safe('media_next')">Next</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAetherLink } from "../../composables/useAetherLink";

const { safe } = useAetherLink();
</script>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 25px;
}

.double-btns {
  display: flex;
  justify-content: center;
  flex-direction: row;
  gap: 0;
}

.double-btns button {
  width: 100%;
}

.btn1 {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}

.btn2 {
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
}

.media-btns {
  display: flex;
  justify-content: center;
  flex-direction: row;
}

button {
  padding: 10px;
  cursor: pointer;
  border-radius: 12px;
  border: 2px solid rgba(216, 50, 60, 0.3);
  background: none;
  color: var(--red);
}
</style>
