<template>
  <div>
    <select>
      <option>bat</option>
      <option>program</option>
      <option>powershell</option>
    </select>
    <select>
      <option>Задача 1</option>
      <option>Задача 2</option>
      <option>Задача 3</option>
    </select>
    <button>Запустить</button>
  </div>
</template>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 50px;
}

button,
select {
  padding: 10px;
  cursor: pointer;
}
</style>
