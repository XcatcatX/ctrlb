import { createButton, createElement } from "./dom.js";
import { parentOf } from "./paths.js";

export function formatSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return value.toFixed(value < 10 ? 1 : 0) + " " + units[unit];
}

export function renderBreadcrumb(container, path, onOpen) {
  container.replaceChildren(createCrumb("Root", "", onOpen));
  let current = "";
  for (const part of path ? path.split("/") : []) {
    current = current ? current + "/" + part : part;
    container.append(createElement("span", "crumb-separator", "/"));
    container.append(createCrumb(part, current, onOpen));
  }
}

function createCrumb(label, target, onOpen) {
  return createButton(label, () => onOpen(target), "crumb");
}

export function renderEntries(listElement, entries, pinnedPaths, actions) {
  const rows = entries.map((entry) =>
    createRow(entry, { isPinnedList: false, pinned: pinnedPaths.has(entry.path) }, actions));
  listElement.replaceChildren(...rows);
}

export function renderPinned(listElement, pins, actions) {
  const rows = pins.map((entry) =>
    createRow(entry, { isPinnedList: true, pinned: true }, actions));
  listElement.replaceChildren(...rows);
}

function createRow(entry, options, actions) {
  const row = createElement("li", options.isPinnedList ? "row row-pinned" : "row");
  row.append(createNameCell(entry, options, actions));
  row.append(createElement("span", "size", entry.is_dir ? "" : formatSize(entry.size)));
  row.append(createButton("Download", () => actions.onDownload(entry), "small"));
  row.append(createButton(options.pinned ? "Unpin" : "Pin", () => actions.onTogglePin(entry), "small"));
  if (!options.isPinnedList) {
    row.append(createButton("Delete", () => actions.onDelete(entry), "small"));
  }
  return row;
}

function createNameCell(entry, options, actions) {
  const cell = createElement("div", "name-cell");
  const label = entry.is_dir ? entry.name + "/" : entry.name;
  const className = entry.is_dir ? "name-link folder" : "name-link";
  cell.append(createButton(label, () => actions.onOpen(entry), className));
  if (entry.is_link) cell.append(createElement("span", "badge", "link"));
  if (options.isPinnedList) {
    const parent = parentOf(entry.path);
    if (parent) cell.append(createElement("span", "location", "in " + parent));
  }
  return cell;
}
