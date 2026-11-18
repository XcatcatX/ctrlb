export class ApiError extends Error {
  constructor(message, status) {
    super(message);
    this.status = status;
  }
}

async function request(url, options) {
  let response;
  try {
    response = await fetch(url, options);
  } catch {
    throw new ApiError("Cannot reach the server.", 0);
  }
  if (!response.ok) {
    const text = await response.text();
    throw new ApiError(text || "Request failed.", response.status);
  }
  return response;
}

function postJson(url, data) {
  return request(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export async function getAuthStatus() {
  return (await request("/api/auth")).json();
}

export async function login(password) {
  await postJson("/api/login", { password });
}

export async function listFolder(path) {
  return (await request("/api/list?path=" + encodeURIComponent(path))).json();
}

export async function createFolder(parentPath, name) {
  await postJson("/api/folder", { path: parentPath, name });
}

export async function deleteEntry(path) {
  await postJson("/api/delete", { path });
}

export async function getPins() {
  return (await request("/api/pins")).json();
}

export async function setPin(path, pinned) {
  await postJson("/api/pins", { path, pinned });
}

export function downloadUrl(path) {
  return "/api/download?path=" + encodeURIComponent(path);
}

// The browser saves the ZIP itself, so big files do not fill the memory.
export function submitZip(paths) {
  document.getElementById("zip-paths").value = JSON.stringify(paths);
  document.getElementById("zip-form").submit();
}

// Sends one part of a file. Returns when the server has saved it.
export function uploadPart(targetPath, offset, blob, onProgress) {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    const query = "path=" + encodeURIComponent(targetPath) + "&offset=" + offset;
    xhr.open("POST", "/api/upload?" + query);
    xhr.upload.onprogress = (event) => onProgress(Math.min(event.loaded, blob.size));
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) resolve();
      else reject(new ApiError(xhr.responseText || "Upload failed.", xhr.status));
    };
    xhr.onerror = () => reject(new ApiError("Connection lost.", 0));
    const form = new FormData();
    form.append("file", blob, "part");
    xhr.send(form);
  });
}
