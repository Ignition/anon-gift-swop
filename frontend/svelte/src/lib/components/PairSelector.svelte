<script lang="ts">
  import { onMount } from 'svelte';
  import { names } from '$lib/stores/NameList';
  import { selection } from '$lib/stores/Selection';
  import { pairs, addPair, removePair } from '$lib/stores/ForbiddenPairs';


  let selectedIndex1: number|null = null;
  let selectedIndex2: number|null = null;
  let disableAddButton: boolean = true;

   $: disableAddButton = (
     selectedIndex1 === null || 
     selectedIndex2 === null || 
     selectedIndex1 === selectedIndex2
   );
</script>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500&display=swap');

  * {
    margin: 0;
    padding: 0;
  }

  .container {
    margin: 10px;
    font-family: 'Roboto', sans-serif;
    display: flex;
    flex-direction: column;
    width: 400px;
    padding: 20px;
    margin: 0 auto;
    border: 2px solid #ccc;
    border-radius: 8px;
    background-color: #f9f9f9;
    max-width: 100%; 
  }

  select {
    padding: 10px;
    font-size: 16px;
    border: 2px solid #ccc;
    border-radius: 4px;
    margin-bottom: 10px;
    width: 100%;
  }

  button {
    padding: 10px;
    font-size: 16px;
    background-color: #4caf50;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    width: 100%;
  }

  button:hover {
    background-color: #45a049;
  }

  button:disabled {
    background-color: #ccc; 
    color: #666; 
    cursor: not-allowed; 
  }

  .pair-list {
    margin-top: 10px;
    min-height: 100px;
    max-height: 200px;
    overflow-y: auto;
    border: 2px solid #ccc;
    border-radius: 4px;
    padding: 10px;
    background-color: white;
    overflow-x: hidden; 
    resize: vertical;
    list-style: none;
  }

  .pair-item {
    display: flex;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #ddd;
    justify-content: space-between;
  }

  .pair-item span:first-child {
    justify-content: left;
  }

  .pair-item span:nth-child(3) {
    justify-content: left;
    margin-right: 10px;

  }

  .pair-item:last-child {
    border-bottom: none;
  }

  .pair-item span {
    font-size: 18px;
    flex: 1;
    color: #333;
    display: flex;
    justify-content: center;
  }

.delete-button {
  background-color: #f44336;
  color: white;
  border: none;
  padding: 6px 10px;
  border-radius: 4px;
  cursor: pointer;
  width: 40px; 
  height: 40px; 
}

.delete-button:hover {
  background-color: #e53935;
}


.pair-name {
  flex: 1; 
  color: #333;
}

</style>



<div class="container">
  <select bind:value={selectedIndex1}>
    <option value={null} disabled selected>Select a name</option>
    {#each $names as name, index }
      <option value={index}>{name}</option>
    {/each}
  </select>

  <select bind:value={selectedIndex2}>
    <option value={null} disabled selected>Select a name</option>
    {#each $names as name, index }
      <option value={index}>{name}</option>
    {/each}
  </select>

  <button on:click={addPair(selectedIndex1,selectedIndex2)} disabled={disableAddButton}>Add Forbidden Pairing</button>

  <ul class="pair-list">
    {#each $pairs as [index1, index2], idx}
      <li class="pair-item">
        <span class="pair-name">{$names[index1]}</span>
        <span class="pair-separator">|</span>
        <span class="pair-name">{$names[index2]}</span>
        <button class="delete-button" on:click={() => removePair(idx)}>🗑️</button>
      </li>
    {/each}
  </ul>
</div>


