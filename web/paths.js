export function joinPath(folder, name) {
  return folder ? folder + "/" + name : name;
}

export function parentOf(path) {
  const index = path.lastIndexOf("/");
  return index === -1 ? "" : path.slice(0, index);
}

export function nameOf(path) {
  return path.slice(path.lastIndexOf("/") + 1);
}
