import { createButton, createElement } from "./dom.js";

// Shows the folders of the main folder in the sidebar.
export function renderSidebar(container, rootEntries, currentPath, onOpen) {
  const folders = rootEntries.filter((entry) => entry.is_dir);
  if (folders.length === 0) {
    container.replaceChildren(createElement("p", "side-empty muted", "No folders"));
    return;
  }
  const buttons = folders.map((entry) => {
    const active = currentPath === entry.path || currentPath.startsWith(entry.path + "/");
    return createButton(entry.name, () => onOpen(entry.path), active ? "side-link active" : "side-link");
  });
  container.replaceChildren(...buttons);
}
