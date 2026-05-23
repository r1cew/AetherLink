<template>
  <div class="app-wrapper">
    <div class="page-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts"></script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: "Segoe UI", sans-serif;
  background: #0c0e0c;
  color: #ffffff;
  font-size: 14px;
}

.app-wrapper {
  min-height: 100vh;
}

.page-content {
  padding-bottom: 70px;
}

@media (min-width: 768px) {
  .page-content {
    padding-bottom: 0;
  }
}
</style>
