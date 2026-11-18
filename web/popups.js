import { createButton, createElement } from "./dom.js";

const layer = document.getElementById("popup-layer");
let escapeHandler = null;

export function closePopup() {
  layer.hidden = true;
  layer.replaceChildren();
  if (escapeHandler) {
    document.removeEventListener("keydown", escapeHandler);
    escapeHandler = null;
  }
}

function showPopup({ title, content, buttons, onCancel }) {
  closePopup();
  const box = createElement("div", "popup");
  box.append(createElement("h2", "", title));
  if (content) box.append(content);
  const row = createElement("div", "popup-buttons");
  for (const button of buttons) {
    row.append(createButton(button.label, button.onClick, button.primary ? "primary" : ""));
  }
  box.append(row);
  layer.append(box);
  layer.hidden = false;
  if (onCancel) {
    escapeHandler = (event) => {
      if (event.key === "Escape") onCancel();
    };
    document.addEventListener("keydown", escapeHandler);
  }
}

function paragraph(text) {
  return createElement("p", "", text);
}

export function showMessage(text, title = "Message") {
  return new Promise((resolve) => {
    const finish = () => {
      closePopup();
      resolve();
    };
    showPopup({
      title,
      content: paragraph(text),
      buttons: [{ label: "OK", primary: true, onClick: finish }],
      onCancel: finish,
    });
  });
}

export function askConfirm(title, text, confirmLabel) {
  return new Promise((resolve) => {
    const finish = (answer) => {
      closePopup();
      resolve(answer);
    };
    showPopup({
      title,
      content: paragraph(text),
      buttons: [
        { label: "Cancel", onClick: () => finish(false) },
        { label: confirmLabel, primary: true, onClick: () => finish(true) },
      ],
      onCancel: () => finish(false),
    });
  });
}

export function askText(title, label) {
  return new Promise((resolve) => {
    const input = createElement("input");
    input.type = "text";
    input.placeholder = label;
    input.setAttribute("aria-label", label);
    const finish = (value) => {
      closePopup();
      resolve(value);
    };
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter") finish(input.value.trim());
    });
    showPopup({
      title,
      content: input,
      buttons: [
        { label: "Cancel", onClick: () => finish(null) },
        { label: "OK", primary: true, onClick: () => finish(input.value.trim()) },
      ],
      onCancel: () => finish(null),
    });
    input.focus();
  });
}

// Shows the password popup. Returns when the login worked.
export function askPassword(tryLogin) {
  return new Promise((resolve) => {
    const input = createElement("input");
    input.type = "password";
    input.placeholder = "Password";
    input.autocomplete = "current-password";
    input.setAttribute("aria-label", "Password");
    const errorText = createElement("p", "error-text");
    const content = createElement("div");
    content.append(input, errorText);
    const submit = async () => {
      try {
        await tryLogin(input.value);
        closePopup();
        resolve();
      } catch (error) {
        errorText.textContent = error.status === 401 ? "Wrong password." : error.message;
        input.select();
      }
    };
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter") submit();
    });
    showPopup({
      title: "Password",
      content,
      buttons: [{ label: "Open", primary: true, onClick: submit }],
    });
    input.focus();
  });
}

export function showProgress(title) {
  const line = paragraph("Starting...");
  const bar = createElement("progress");
  bar.max = 1;
  bar.value = 0;
  const detail = createElement("p", "muted");
  const content = createElement("div");
  content.append(line, bar, detail);
  showPopup({ title, content, buttons: [] });
  return {
    update(lineText, detailText, fraction) {
      line.textContent = lineText;
      detail.textContent = detailText;
      bar.value = fraction;
    },
    close: closePopup,
  };
}
