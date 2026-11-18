import { uploadPart } from "./api.js";
import { joinPath } from "./paths.js";

// Parts are smaller than 100 MB, because the tunnel does not accept bigger requests.
const PART_SIZE = 50 * 1024 * 1024;

// Items from the file picker. Folders keep their path.
export function itemsFromFileList(fileList) {
  return [...fileList].map((file) => ({ file, path: file.webkitRelativePath || file.name }));
}

// Items from a drop. Must be called directly in the drop event.
export async function itemsFromDrop(dataTransfer) {
  const entries = [...dataTransfer.items].map((item) => item.webkitGetAsEntry()).filter(Boolean);
  const result = [];
  for (const entry of entries) await collectEntry(entry, "", result);
  return result;
}

async function collectEntry(entry, parentPath, result) {
  const path = parentPath + entry.name;
  if (entry.isFile) {
    const file = await new Promise((resolve, reject) => entry.file(resolve, reject));
    result.push({ file, path });
  } else if (entry.isDirectory) {
    const children = await readAllEntries(entry.createReader());
    for (const child of children) await collectEntry(child, path + "/", result);
  }
}

async function readAllEntries(reader) {
  const all = [];
  while (true) {
    const batch = await new Promise((resolve, reject) => reader.readEntries(resolve, reject));
    if (batch.length === 0) return all;
    all.push(...batch);
  }
}

// Uploads all items into the folder. onProgress gets info about the current state.
export async function uploadItems(items, folderPath, onProgress) {
  const total = items.reduce((sum, item) => sum + item.file.size, 0);
  let done = 0;
  for (let index = 0; index < items.length; index++) {
    const { file, path } = items[index];
    const report = (sentInFile) =>
      onProgress({ index: index + 1, count: items.length, name: file.name, sent: done + sentInFile, total });
    await uploadInParts(file, joinPath(folderPath, path), report);
    done += file.size;
  }
}

async function uploadInParts(file, targetPath, report) {
  let offset = 0;
  do {
    const end = Math.min(offset + PART_SIZE, file.size);
    const start = offset;
    await uploadPart(targetPath, start, file.slice(start, end), (loaded) => report(start + loaded));
    offset = end;
  } while (offset < file.size);
  report(file.size);
}
