<template>
  <nav class="navigation">
    <div class="nav-header">
      <h2 class="nav-title">Меню</h2>
      <div class="nav-indicator"></div>
    </div>

    <div class="nav-grid">
      <router-link to="/" class="nav-card" active-class="active">
        <div class="card-strip"></div>
        <div class="card-icon">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2h-5v-7H9v7H4a2 2 0 0 1-2-2z"
            />
          </svg>
        </div>
        <div class="card-info">
          <span class="card-title">Главная</span>
          <span class="card-description">Управление устройствами</span>
        </div>
        <div class="card-arrow">
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M9 18l6-6-6-6" />
          </svg>
        </div>
      </router-link>

      <router-link to="/tasks" class="nav-card" active-class="active">
        <div class="card-strip"></div>
        <div class="card-icon">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"
            />
            <rect x="9" y="3" width="6" height="4" rx="1" />
            <path d="M9 12h6" />
            <path d="M9 16h6" />
          </svg>
        </div>
        <div class="card-info">
          <span class="card-title">Задачи</span>
          <span class="card-description">Управление заданиями</span>
        </div>
        <div class="card-arrow">
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M9 18l6-6-6-6" />
          </svg>
        </div>
      </router-link>
    </div>

    <div class="nav-footer">
      <div class="version">AetherLink v1.0.0</div>
    </div>
  </nav>
</template>

<script>
import "../assets/navigation.css";
</script>
