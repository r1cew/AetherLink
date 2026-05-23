<template>
  <div>
    <div class="input-block">
      <input />
      <button>+</button>
    </div>
    <select>
      <option>Задача 1</option>
      <option>Задача 2</option>
      <option>Задача 3</option>
    </select>
    <div class="history">
      <p>Ваши последние команды:</p>
      <div>
        <button>Задача 1</button>
        <button>Задача 2</button>
        <button>Задача 3</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
div {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

button,
select,
input {
  padding: 10px;
  cursor: pointer;
}

.input-block {
  display: flex;
  justify-content: center;
  flex-direction: row;
  gap: 0;
}

.history {
  display: flex;
  align-items: center;
}

.history div {
  display: flex;
  gap: 10px;
}

.history div button {
  width: 75vw;
}

.input-block input {
  width: 70vw;
}

.input-block button {
  width: 20vw;
}
</style>
