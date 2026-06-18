import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

document.addEventListener('DOMContentLoaded', () => {
    console.log('Tauri internals:', window.__TAURI_INTERNALS__);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

