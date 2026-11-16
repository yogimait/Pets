import { mount } from "svelte";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app.css";
import App from "./App.svelte";
import OrchestratorWindow from "./lib/components/OrchestratorWindow.svelte";
import PetBubble from "./lib/components/PetBubble.svelte";
import PetPen from "./lib/components/PetPen.svelte";
import PetSprite from "./lib/components/PetSprite.svelte";

const label = isTauri() ? getCurrentWindow().label : "main";
const Root =
  label === "orchestrator"
    ? OrchestratorWindow
    : label === "pet-pen"
      ? PetPen
      : label === "pet-bubble"
        ? PetBubble
        : label.startsWith("pet-")
          ? PetSprite
          : App;

const app = mount(Root, { target: document.getElementById("app")! });

export default app;
