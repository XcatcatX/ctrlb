import * as api from "./api.js";
import { askConfirm, askPassword, askText, showMessage, showProgress } from "./popups.js";
import { formatSize, renderBreadcrumb, renderEntries, renderPinned } from "./list.js";
import { renderSidebar } from "./sidebar.js";
import { itemsFromDrop, itemsFromFileList, uploadItems } from "./upload.js";
import { nameOf } from "./paths.js";

const byId = (id) => document.getElementById(id);
const sidebar = byId("sidebar");
const sideBackdrop = byId("side-backdrop");
const sideFolders = byId("side-folders");
const navRoot = byId("nav-root");
const pageTitle = byId("page-title");
const pageSubtitle = byId("page-subtitle");
const pinnedSection = byId("pinned-section");
const pinnedList = byId("pinned-list");
const pinnedCount = byId("pinned-count");
const breadcrumb = byId("breadcrumb");
const itemCount = byId("item-count");
const fileList = byId("file-list");
const emptyMessage = byId("empty-message");
const dropOverlay = byId("drop-overlay");
const fileInput = byId("file-input");
const folderInput = byId("folder-input");

let currentPath = "";
let pinnedPaths = new Set();

const rowActions = {
  onOpen: openEntry,
  onDownload: downloadEntry,
  onTogglePin: togglePin,
  onDelete: deleteEntry,
};

// ----- Folder navigation (the folder is kept in the address bar) -----

function readHash() {
  try {
    return decodeURIComponent(location.hash.slice(1));
  } catch {
    return "";
  }
}

function goTo(path) {
  setMenuOpen(false);
  const newHash = encodeURIComponent(path);
  if (location.hash.slice(1) === newHash) {
    currentPath = path;
    refresh();
  } else {
    location.hash = newHash;
  }
}

window.addEventListener("hashchange", () => {
  currentPath = readHash();
  refresh();
});

function setMenuOpen(open) {
  sidebar.classList.toggle("open", open);
  sideBackdrop.hidden = !open;
}

// ----- Loading and drawing -----

async function refresh() {
  try {
    const [listing, pins] = await Promise.all([api.listFolder(currentPath), api.getPins()]);
    currentPath = listing.path;
    const rootEntries = currentPath === "" ? listing.entries : (await api.listFolder("")).entries;
    pinnedPaths = new Set(pins.map((pin) => pin.path));
    draw(listing.entries, pins, rootEntries);
  } catch (error) {
    await handleError(error);
  }
}

function draw(entries, pins, rootEntries) {
  renderSidebar(sideFolders, rootEntries, currentPath, goTo);
  navRoot.classList.toggle("active", currentPath === "");
  renderBreadcrumb(breadcrumb, currentPath, goTo);
  renderEntries(fileList, entries, pinnedPaths, rowActions);
  renderPinned(pinnedList, pins, rowActions);
  pageTitle.textContent = currentPath ? nameOf(currentPath) : "Files";
  pageSubtitle.textContent = describeFolder(entries);
  itemCount.textContent = entries.length === 1 ? "1 item" : `${entries.length} items`;
  pinnedCount.textContent = String(pins.length);
  pinnedSection.hidden = pins.length === 0;
  emptyMessage.hidden = entries.length > 0;
}

function describeFolder(entries) {
  const folders = entries.filter((entry) => entry.is_dir).length;
  const files = entries.length - folders;
  const total = entries.reduce((sum, entry) => sum + entry.size, 0);
  return `${folders} folders, ${files} files, ${formatSize(total)}`;
}

async function handleError(error) {
  if (error.status === 401) {
    await askPassword(api.login);
    await refresh();
  } else if (error.status === 404 && currentPath !== "") {
    goTo("");
  } else {
    await showMessage(error.message || "Something went wrong.", "Error");
  }
}

// ----- Actions -----

function startDownload(url) {
  const link = document.createElement("a");
  link.href = url;
  link.download = "";
  document.body.append(link);
  link.click();
  link.remove();
}

function openEntry(entry) {
  if (entry.is_dir) goTo(entry.path);
  else startDownload(api.downloadUrl(entry.path));
}

// A file is downloaded as it is. Only a folder is packed as a ZIP.
function downloadEntry(entry) {
  if (entry.is_dir) api.submitZip([entry.path]);
  else startDownload(api.downloadUrl(entry.path));
}

async function togglePin(entry) {
  try {
    await api.setPin(entry.path, !pinnedPaths.has(entry.path));
    await refresh();
  } catch (error) {
    await handleError(error);
  }
}

function deleteQuestion(entry) {
  if (entry.is_link) {
    return `Remove the link "${entry.name}"? The linked ${entry.is_dir ? "folder" : "file"} stays untouched.`;
  }
  if (entry.is_dir) {
    return `Delete the folder "${entry.name}" and everything in it? This cannot be undone.`;
  }
  return `Delete "${entry.name}"? This cannot be undone.`;
}

async function deleteEntry(entry) {
  const confirmed = await askConfirm("Delete", deleteQuestion(entry), entry.is_link ? "Remove" : "Delete");
  if (!confirmed) return;
  try {
    await api.deleteEntry(entry.path);
    await refresh();
  } catch (error) {
    await handleError(error);
  }
}

async function createFolder() {
  const name = await askText("New folder", "Folder name");
  if (!name) return;
  try {
    await api.createFolder(currentPath, name);
    await refresh();
  } catch (error) {
    await handleError(error);
  }
}

async function handleUpload(items) {
  if (items.length === 0) return;
  const progress = showProgress("Uploading");
  let failure = null;
  try {
    await uploadItems(items, currentPath, (info) => {
      const fraction = info.total > 0 ? info.sent / info.total : 1;
      progress.update(
        `File ${info.index} of ${info.count}: ${info.name}`,
        `${formatSize(info.sent)} of ${formatSize(info.total)}`,
        fraction,
      );
    });
  } catch (error) {
    failure = error;
  }
  progress.close();
  await refresh();
  if (failure && failure.status !== 401) {
    await showMessage(failure.message, "Upload failed");
  }
}

// ----- Buttons and drag and drop -----

function connectButtons() {
  byId("btn-menu").addEventListener("click", () => setMenuOpen(!sidebar.classList.contains("open")));
  sideBackdrop.addEventListener("click", () => setMenuOpen(false));
  navRoot.addEventListener("click", () => goTo(""));
  byId("btn-upload-files").addEventListener("click", () => fileInput.click());
  byId("btn-upload-folder").addEventListener("click", () => folderInput.click());
  byId("btn-new-folder").addEventListener("click", createFolder);
  for (const input of [fileInput, folderInput]) {
    input.addEventListener("change", () => {
      const items = itemsFromFileList(input.files);
      input.value = "";
      handleUpload(items);
    });
  }
}

function hasFiles(event) {
  return event.dataTransfer && [...event.dataTransfer.types].includes("Files");
}

function connectDragAndDrop() {
  let depth = 0;
  window.addEventListener("dragenter", (event) => {
    if (!hasFiles(event)) return;
    event.preventDefault();
    depth++;
    dropOverlay.hidden = false;
  });
  window.addEventListener("dragover", (event) => {
    if (hasFiles(event)) event.preventDefault();
  });
  window.addEventListener("dragleave", (event) => {
    if (!hasFiles(event)) return;
    depth = Math.max(0, depth - 1);
    if (depth === 0) dropOverlay.hidden = true;
  });
  window.addEventListener("drop", async (event) => {
    if (!hasFiles(event)) return;
    event.preventDefault();
    depth = 0;
    dropOverlay.hidden = true;
    handleUpload(await itemsFromDrop(event.dataTransfer));
  });
}

// ----- Start -----

async function start() {
  connectButtons();
  connectDragAndDrop();
  currentPath = readHash();
  try {
    const status = await api.getAuthStatus();
    if (status.password_required && !status.logged_in) await askPassword(api.login);
  } catch (error) {
    await showMessage(error.message, "Error");
  }
  await refresh();
}

start();
