<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import TdCanvas from "../lib/components/TdCanvas.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { event } from "@tauri-apps/api";
  let name = "";
  let greetMsg = "";
  let points = [15];
  let path: string = "";
  let interval_set = false;
  let volume = 1;

  $: volume, set_volume();
  async function play_from_path() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    await invoke("play", { path: path });
    if (interval_set == false) {
      let inter = setInterval(function () {
        get_points();
      }, 20 * 1);
      interval_set = true;
    }
  }

  async function set_volume() {
    await invoke("set_volume", { volume: volume / 100 });
  }

  async function get_points() {
    points = await invoke("get_points", {});
  }

  listen("tauri://file-drop", (event) => {
    let payload = event.payload;
    let array_path = typeof payload === typeof ["string"] ? payload : undefined;
    if (array_path != null && array_path != undefined) {
      path = array_path.toString();
    }
  });
</script>

<div class="container">
  <h1>Music Visualiser</h1>

  <div class="td-container">
    <TdCanvas {points}></TdCanvas>
  </div>

  <button on:click={play_from_path}
    >Play file : {path.split("\\").at(-1)?.split(".").at(0)}</button
  >

  <input type="range" bind:value={volume} />
</div>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  .row {
    display: flex;
    justify-content: center;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
